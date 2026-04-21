//! Screen capture module using Windows Graphics Capture (WGC) API.
//!
//! Uses a tick-based design inspired by OBS:
//! - WGC captures frames into a "latest frame" slot (variable rate)
//! - A separate ticker thread pulls the latest frame at exactly 60fps
//! - This ensures FFmpeg always receives frames at a precise constant rate
//! - If no new frame is available, the previous frame is duplicated
//!
//! This produces perfectly-timed video regardless of monitor refresh rate.

use std::io::Write;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use windows_capture::capture::{Context, GraphicsCaptureApiHandler};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::monitor::Monitor;
use windows_capture::settings::{
    ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings,
    MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
};

/// Shared latest-frame buffer. The capture handler writes here,
/// the ticker thread reads from here.
pub struct FrameSlot {
    /// The latest captured frame data (BGRA pixels)
    data: Vec<u8>,
    /// Whether a new frame has been written since last read
    fresh: bool,
}

/// Flags passed through Settings to the capture handler.
pub struct CaptureFlags {
    pub width: u32,
    pub height: u32,
    pub frame_slot: Arc<Mutex<FrameSlot>>,
    pub stop_signal: Arc<AtomicBool>,
}

/// The capture handler — writes frames into the shared slot as fast as WGC delivers them.
struct ClipSyncCapture {
    frame_slot: Arc<Mutex<FrameSlot>>,
    stop_signal: Arc<AtomicBool>,
    width: u32,
    height: u32,
    frame_size: usize,
}

impl GraphicsCaptureApiHandler for ClipSyncCapture {
    type Flags = CaptureFlags;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        let frame_size = (ctx.flags.width * ctx.flags.height * 4) as usize;
        println!(
            "[ClipSync Capture] Handler initialized ({}x{}, {} bytes/frame)",
            ctx.flags.width, ctx.flags.height, frame_size
        );
        Ok(Self {
            frame_slot: ctx.flags.frame_slot,
            stop_signal: ctx.flags.stop_signal,
            width: ctx.flags.width,
            height: ctx.flags.height,
            frame_size,
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        if self.stop_signal.load(Ordering::Relaxed) {
            capture_control.stop();
            return Ok(());
        }

        // Extract raw BGRA pixels
        let mut buffer = frame.buffer()?;
        let raw_data = buffer.as_raw_buffer();

        if raw_data.len() >= self.frame_size {
            // Write into the shared slot — the ticker thread will read it
            if let Ok(mut slot) = self.frame_slot.try_lock() {
                slot.data.clear();
                slot.data.extend_from_slice(&raw_data[..self.frame_size]);
                slot.fresh = true;
            }
            // If we can't lock, skip — the ticker is reading, we'll get the next one
        }

        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("[ClipSync Capture] Capture session closed");
        Ok(())
    }
}

/// Configuration for the capture session.
pub struct CaptureConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
}

/// Start the capture pipeline.
///
/// This spawns two threads:
/// 1. **Capture thread**: WGC captures frames into a shared slot (variable rate)
/// 2. **Ticker thread**: Reads the slot at exactly `fps` Hz, writes to FFmpeg stdin
///
/// The ticker thread is what ensures constant, accurate frame timing.
///
/// Returns a JoinHandle for the capture thread. The ticker runs
/// until the stop signal is set.
pub fn start_capture(
    config: &CaptureConfig,
    mut ffmpeg_stdin: std::process::ChildStdin,
    stop_signal: Arc<AtomicBool>,
    frame_count: Arc<std::sync::atomic::AtomicU64>,
) -> Result<(), Box<dyn std::error::Error>> {
    let width = config.width;
    let height = config.height;
    let fps = config.fps;
    let frame_size = (width * height * 4) as usize;
    let frame_interval = Duration::from_nanos(1_000_000_000 / fps as u64);

    println!(
        "[ClipSync Capture] Starting ({}x{} @ {}fps, interval {:?})",
        width, height, fps, frame_interval
    );

    // Shared frame slot — capture writes, ticker reads
    let frame_slot = Arc::new(Mutex::new(FrameSlot {
        data: vec![0u8; frame_size], // Start with a black frame
        fresh: false,
    }));

    let slot_for_capture = frame_slot.clone();
    let stop_for_capture = stop_signal.clone();

    // ── Ticker thread: reads frames at exactly target FPS ────────
    // This is the critical thread that ensures correct timing.
    let slot_for_ticker = frame_slot.clone();
    let stop_for_ticker = stop_signal.clone();

    let ticker_handle = std::thread::Builder::new()
        .name("clipsync-ticker".into())
        .spawn(move || {
            let mut next_tick = Instant::now();
            let mut frames_sent: u64 = 0;
            let mut duplicated: u64 = 0;
            let start_time = Instant::now();

            while !stop_for_ticker.load(Ordering::Relaxed) {
                // Read the latest frame from the slot
                let frame_data = {
                    let mut slot = slot_for_ticker.lock().unwrap();
                    if !slot.fresh {
                        duplicated += 1;
                    }
                    slot.fresh = false;
                    slot.data.clone()
                };

                // Write to FFmpeg stdin — exactly one frame per tick
                if frame_data.len() == frame_size {
                    if let Err(e) = ffmpeg_stdin.write_all(&frame_data) {
                        eprintln!("[ClipSync Ticker] Write error: {}", e);
                        break;
                    }
                    frames_sent += 1;
                    frame_count.store(frames_sent, Ordering::Relaxed);

                    if frames_sent % 300 == 0 {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let actual_fps = frames_sent as f64 / elapsed;
                        println!(
                            "[ClipSync Ticker] {} frames sent ({:.1} fps), {} duplicated",
                            frames_sent, actual_fps, duplicated
                        );
                    }
                }

                // Sleep until next tick — this is what guarantees constant FPS
                next_tick += frame_interval;
                let now = Instant::now();
                if next_tick > now {
                    std::thread::sleep(next_tick - now);
                } else {
                    // We're behind — reset to avoid trying to catch up
                    next_tick = now + frame_interval;
                }
            }

            // Drop stdin to signal FFmpeg that input is done
            drop(ffmpeg_stdin);
            println!(
                "[ClipSync Ticker] Stopped ({} frames sent, {} duplicated)",
                frames_sent, duplicated
            );
        })?;

    // ── Capture thread: WGC writes frames into slot ──────────────
    let monitor = Monitor::primary()?;

    let flags = CaptureFlags {
        width,
        height,
        frame_slot: slot_for_capture,
        stop_signal: stop_for_capture,
    };

    let settings = Settings::new(
        monitor,
        CursorCaptureSettings::WithoutCursor,
        DrawBorderSettings::WithoutBorder,
        SecondaryWindowSettings::Default,
        MinimumUpdateIntervalSettings::Default,
        DirtyRegionSettings::Default,
        ColorFormat::Bgra8,
        flags,
    );

    // This blocks until capture is stopped
    ClipSyncCapture::start(settings)?;

    // Wait for ticker to finish
    let _ = ticker_handle.join();

    println!("[ClipSync Capture] Pipeline finished");
    Ok(())
}
