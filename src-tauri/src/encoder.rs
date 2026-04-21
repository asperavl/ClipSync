//! FFmpeg encoding module using ffmpeg-sidecar (subprocess approach).
//!
//! Spawns an FFmpeg process configured for NVENC CQP 20 encoding
//! with the segment muxer for replay buffer support.
//! Accepts video via stdin and audio via a Windows named pipe.

use std::process::ChildStdin;

use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::command::FfmpegCommand;

use crate::audio::AUDIO_PIPE_NAME;

/// Encoder configuration.
pub struct EncoderConfig {
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    pub qp: u32,
    /// Pattern for segment output, e.g. "/tmp/clipsync_buffer/seg_%05d.mp4"
    pub segment_pattern: String,
    /// Duration of each segment in seconds
    pub segment_duration: u32,
    /// Whether to include audio input from the named pipe
    pub enable_audio: bool,
    /// Audio sample rate
    pub audio_sample_rate: u32,
    /// Audio channels
    pub audio_channels: u16,
}

impl Default for EncoderConfig {
    fn default() -> Self {
        Self {
            width: 1920,
            height: 1080,
            fps: 60,
            qp: 20,
            segment_pattern: String::from("seg_%05d.mp4"),
            segment_duration: 2,
            enable_audio: true,
            audio_sample_rate: 48000,
            audio_channels: 2,
        }
    }
}

/// A running FFmpeg encoder process.
pub struct Encoder {
    child: FfmpegChild,
}

impl Encoder {
    /// Spawn FFmpeg for segment encoding with audio + video.
    ///
    /// Video: rawvideo BGRA from stdin (pipe:0)
    /// Audio: raw f32le PCM from named pipe (\\.\pipe\clipsync_audio)
    pub fn start(config: &EncoderConfig) -> Result<(Self, ChildStdin), Box<dyn std::error::Error>> {
        let video_size = format!("{}x{}", config.width, config.height);
        let fps_str = config.fps.to_string();
        let gop_size = (config.fps * config.segment_duration).to_string();
        let qp_str = config.qp.to_string();
        let seg_time = config.segment_duration.to_string();
        let audio_rate = config.audio_sample_rate.to_string();
        let audio_ch = config.audio_channels.to_string();

        println!(
            "[ClipSync Encoder] Starting FFmpeg ({}x{} @ {}fps, CQP {}, {}s segments, audio: {})",
            config.width,
            config.height,
            config.fps,
            config.qp,
            config.segment_duration,
            if config.enable_audio { "enabled" } else { "disabled" }
        );

        let use_nvenc = check_nvenc_available();

        let mut cmd = FfmpegCommand::new();
        cmd.args(["-hide_banner", "-loglevel", "warning"]);

        // Input 0: raw video from stdin
        cmd.args(["-f", "rawvideo"])
            .args(["-pixel_format", "bgra"])
            .args(["-video_size", &video_size])
            .args(["-framerate", &fps_str])
            .args(["-i", "pipe:0"]);

        // Input 1: raw audio from named pipe (WASAPI loopback)
        if config.enable_audio {
            cmd.args(["-f", "f32le"])
                .args(["-ar", &audio_rate])
                .args(["-ac", &audio_ch])
                .args(["-i", AUDIO_PIPE_NAME]);
        }

        // Video encoding
        if use_nvenc {
            println!("[ClipSync Encoder] Using NVIDIA NVENC hardware encoder");
            cmd.args([
                "-c:v", "h264_nvenc",
                "-qp", &qp_str,
                "-profile:v", "high",
                "-g", &gop_size,
                "-bf", "0",
                "-preset", "p4",
            ]);
        } else {
            println!("[ClipSync Encoder] Falling back to libx264 software encoder");
            cmd.args([
                "-c:v", "libx264",
                "-crf", &qp_str,
                "-profile:v", "high",
                "-g", &gop_size,
                "-bf", "0",
                "-preset", "fast",
            ]);
        }

        // Audio encoding (if enabled)
        if config.enable_audio {
            cmd.args(["-c:a", "aac", "-b:a", "192k"]);
        }

        // Output: segment muxer
        cmd.args(["-pix_fmt", "yuv420p"])
            .args(["-f", "segment"])
            .args(["-segment_time", &seg_time])
            .args(["-reset_timestamps", "1"])
            .arg(&config.segment_pattern);

        let mut child = cmd.spawn()?;

        let stdin = child
            .take_stdin()
            .ok_or("Failed to take FFmpeg stdin pipe")?;

        println!("[ClipSync Encoder] FFmpeg process started");

        Ok((Self { child }, stdin))
    }

    /// Wait for FFmpeg to finish encoding.
    pub fn finish(mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[ClipSync Encoder] Waiting for FFmpeg to finish...");
        let _output = self.child.wait()?;
        println!("[ClipSync Encoder] FFmpeg finished.");
        Ok(())
    }

    /// Kill the FFmpeg process immediately.
    #[allow(dead_code)]
    pub fn kill(mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[ClipSync Encoder] Killing FFmpeg process...");
        self.child.kill()?;
        Ok(())
    }
}

/// Check if NVENC is available.
fn check_nvenc_available() -> bool {
    let output = std::process::Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            stdout.contains("h264_nvenc")
        }
        Err(_) => false,
    }
}
