//! Audio capture module using WASAPI loopback.
//!
//! Captures desktop audio (whatever is playing through speakers/headphones)
//! using Windows Audio Session API (WASAPI) in loopback mode.
//! This is the same method used by OBS, ShadowPlay, and Medal.tv.
//!
//! Audio samples are written to a Windows named pipe that FFmpeg reads from.

use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use wasapi::*;

/// Audio capture configuration.
pub struct AudioConfig {
    /// Sample rate in Hz (e.g. 48000)
    pub sample_rate: u32,
    /// Number of channels (typically 2 for stereo)
    pub channels: u16,
    /// Bits per sample (32 for float)
    pub bits_per_sample: u16,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48000,
            channels: 2,
            bits_per_sample: 32,
        }
    }
}

/// The named pipe path for audio data on Windows.
pub const AUDIO_PIPE_NAME: &str = r"\\.\pipe\clipsync_audio";

/// Start WASAPI loopback audio capture.
///
/// This captures whatever audio is playing on the system's default output
/// device (speakers/headphones) and writes raw PCM samples to a named pipe.
///
/// **Blocks the calling thread.** Run on a dedicated thread.
pub fn start_audio_capture(
    config: &AudioConfig,
    stop_signal: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Initialize COM for this thread
    initialize_mta().ok();

    // Get the default render (output) device — this is what the user hears
    let enumerator = DeviceEnumerator::new()?;
    let device = enumerator.get_default_device(&Direction::Render)?;
    let device_name = device.get_friendlyname()?;
    println!(
        "[ClipSync Audio] Using output device: {} (loopback mode)",
        device_name
    );

    // Set up audio client for loopback capture
    let mut audio_client = device.get_iaudioclient()?;

    // Request format: 32-bit float, 48kHz, stereo
    let desired_format = WaveFormat::new(
        config.bits_per_sample as usize,
        config.bits_per_sample as usize,
        &SampleType::Float,
        config.sample_rate as usize,
        config.channels as usize,
        None,
    );

    let blockalign = desired_format.get_blockalign() as usize;

    println!(
        "[ClipSync Audio] Format: {}Hz, {}ch, {}-bit float, blockalign={}",
        config.sample_rate, config.channels, config.bits_per_sample, blockalign
    );

    let (def_time, min_time) = audio_client.get_device_period()?;
    println!(
        "[ClipSync Audio] Device period: default={}00ns, min={}00ns",
        def_time, min_time
    );

    // Initialize in loopback capture mode
    // Direction::Capture with loopback enabled on a render device
    let mode = StreamMode::EventsShared {
        autoconvert: true,
        buffer_duration_hns: def_time,
    };
    audio_client.initialize_client(&desired_format, &Direction::Capture, &mode)?;
    println!("[ClipSync Audio] Loopback capture initialized");

    let h_event = audio_client.set_get_eventhandle()?;
    let capture_client = audio_client.get_audiocaptureclient()?;

    // Create the named pipe — FFmpeg will read from this
    let mut pipe = open_audio_pipe()?;
    println!("[ClipSync Audio] Named pipe opened, FFmpeg connected");

    // Start the audio stream
    audio_client.start_stream()?;
    println!("[ClipSync Audio] Capture started");

    let mut sample_queue: VecDeque<u8> = VecDeque::with_capacity(blockalign * 4096);
    let mut total_bytes: u64 = 0;
    let start_time = std::time::Instant::now();

    // Capture loop
    loop {
        if stop_signal.load(Ordering::Relaxed) {
            println!("[ClipSync Audio] Stop signal received");
            break;
        }

        // Wait for audio data to be available (100ms timeout)
        if h_event.wait_for_event(100).is_err() {
            continue;
        }

        // Read available audio data into our queue
        let _info = capture_client.read_from_device_to_deque(&mut sample_queue)?;

        // Write all available data to the pipe
        while sample_queue.len() >= blockalign {
            // Drain a chunk from the queue
            let chunk_size = sample_queue.len().min(blockalign * 1024);
            let chunk: Vec<u8> = sample_queue.drain(..chunk_size).collect();

            if let Err(e) = pipe.write_all(&chunk) {
                eprintln!("[ClipSync Audio] Pipe write error: {}", e);
                stop_signal.store(true, Ordering::Relaxed);
                break;
            }
            total_bytes += chunk.len() as u64;
        }

        // Periodic status log
        let elapsed = start_time.elapsed().as_secs();
        if elapsed > 0 && elapsed % 10 == 0 && total_bytes > 0 {
            let mb = total_bytes as f64 / (1024.0 * 1024.0);
            println!(
                "[ClipSync Audio] {}s captured, {:.1} MB written",
                elapsed, mb
            );
        }
    }

    audio_client.stop_stream()?;
    drop(pipe);
    println!(
        "[ClipSync Audio] Capture stopped ({:.1} MB total)",
        total_bytes as f64 / (1024.0 * 1024.0)
    );

    Ok(())
}

/// Create a Windows named pipe for audio data.
fn open_audio_pipe() -> Result<File, Box<dyn std::error::Error>> {
    use std::os::windows::io::FromRawHandle;
    use std::ptr;

    const PIPE_ACCESS_OUTBOUND: u32 = 0x00000002;
    const PIPE_TYPE_BYTE: u32 = 0x00000000;
    const PIPE_WAIT: u32 = 0x00000000;
    const PIPE_UNLIMITED_INSTANCES: u32 = 255;
    const BUFFER_SIZE: u32 = 65536;
    const INVALID_HANDLE_VALUE: isize = -1;

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateNamedPipeA(
            name: *const u8,
            open_mode: u32,
            pipe_mode: u32,
            max_instances: u32,
            out_buffer_size: u32,
            in_buffer_size: u32,
            default_timeout: u32,
            security_attrs: *const u8,
        ) -> isize;
    }

    let pipe_name = format!("{}\0", AUDIO_PIPE_NAME);

    let handle = unsafe {
        CreateNamedPipeA(
            pipe_name.as_ptr(),
            PIPE_ACCESS_OUTBOUND,
            PIPE_TYPE_BYTE | PIPE_WAIT,
            PIPE_UNLIMITED_INSTANCES,
            BUFFER_SIZE,
            BUFFER_SIZE,
            0,
            ptr::null(),
        )
    };

    if handle == INVALID_HANDLE_VALUE {
        return Err("Failed to create named pipe".into());
    }

    // Wait for FFmpeg to connect to the pipe
    #[link(name = "kernel32")]
    extern "system" {
        fn ConnectNamedPipe(pipe: isize, overlapped: *const u8) -> i32;
    }

    println!("[ClipSync Audio] Waiting for FFmpeg to connect to audio pipe...");
    let _connected = unsafe { ConnectNamedPipe(handle, ptr::null()) };
    // ConnectNamedPipe returns 0 on success in overlapped mode,
    // and ERROR_PIPE_CONNECTED (535) if already connected — both are fine.

    Ok(unsafe { File::from_raw_handle(handle as *mut std::ffi::c_void) })
}

/// List available audio output devices for the settings UI.
pub fn list_audio_devices() -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    initialize_mta().ok();
    let enumerator = DeviceEnumerator::new()?;
    let collection = enumerator.get_device_collection(&Direction::Render)?;

    let mut devices = Vec::new();
    for device in collection.into_iter() {
        if let Ok(device) = device {
            let name = device.get_friendlyname().unwrap_or_default();
            let id = device.get_id().unwrap_or_default();
            devices.push((id, name));
        }
    }

    Ok(devices)
}
