# FFmpeg Sidecar — Quick Reference

**Crate**: `ffmpeg-sidecar`
**Source**: https://github.com/nathanbabcock/ffmpeg-sidecar
**Approach**: Spawns FFmpeg binary as subprocess, communicates via stdin/stdout pipes.

## Installation
```toml
[dependencies]
ffmpeg-sidecar = "2"
```

## Why Sidecar Over C Bindings (`ffmpeg-next`)
- Zero build complexity (no C library linking, no FFMPEG_DIR, no pkg-config)
- Same NVENC encoder, same quality, same parameters
- FFmpeg CLI args are extremely well-documented
- Easy to bundle with Tauri sidecar for distribution
- The user already has FFmpeg installed

---

## ClipSync Encoding Command

### Full command (what we pipe to):
```
ffmpeg
  -f rawvideo                    # Input format: raw video frames
  -pixel_format bgra             # Pixel format from windows-capture
  -video_size 1920x1080          # Resolution
  -framerate 60                  # FPS
  -i pipe:0                     # Read from stdin

  -c:v h264_nvenc                # NVIDIA hardware encoder
  -qp 20                        # CQP mode, quality level 20
  -profile:v high                # H.264 High profile
  -g 120                         # GOP size (2 seconds at 60fps)
  -bf 0                          # No B-frames (simpler ring buffer)
  -preset p4                     # NVENC quality/speed preset
  -pix_fmt yuv420p               # Output pixel format

  -y output.mp4                  # Output file (overwrite)
```

### Parameter breakdown:
| Param | Value | Why |
|---|---|---|
| `-f rawvideo` | raw frames | We pipe uncompressed BGRA frames from capture |
| `-pixel_format bgra` | BGRA | windows-capture outputs BGRA (or RGBA, configurable) |
| `-video_size WxH` | 1920x1080 | Must match capture resolution |
| `-framerate N` | 60 | Must match capture FPS |
| `-i pipe:0` | stdin | Frames piped from Rust process |
| `-c:v h264_nvenc` | NVENC | Hardware H.264 encoding |
| `-qp 20` | CQP 20 | Fixed quality, visually excellent |
| `-profile:v high` | High | Best compression efficiency for H.264 |
| `-g 120` | 2s GOP | Keyframe every 2 seconds (FPS × 2) |
| `-bf 0` | No B-frames | Cleaner ring buffer cuts |
| `-preset p4` | Quality preset | Balance of speed and quality |
| `-pix_fmt yuv420p` | YUV 4:2:0 | Standard for H.264, max compatibility |

---

## Rust Usage Pattern

### Basic: Pipe raw frames to FFmpeg

```rust
use std::io::Write;
use ffmpeg_sidecar::command::FfmpegCommand;

fn encode_frames(
    width: u32,
    height: u32,
    fps: u32,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let gop_size = fps * 2; // Keyframe every 2 seconds

    let mut ffmpeg = FfmpegCommand::new()
        // Input: raw BGRA frames from stdin
        .args([
            "-f", "rawvideo",
            "-pixel_format", "bgra",
            "-video_size", &format!("{}x{}", width, height),
            "-framerate", &fps.to_string(),
            "-i", "pipe:0",
        ])
        // Encoding: NVENC H.264 CQP 20
        .args([
            "-c:v", "h264_nvenc",
            "-qp", "20",
            "-profile:v", "high",
            "-g", &gop_size.to_string(),
            "-bf", "0",
            "-preset", "p4",
            "-pix_fmt", "yuv420p",
        ])
        // Output
        .args(["-y", output_path])
        .spawn()?;

    // Get stdin handle to pipe frames
    let stdin = ffmpeg.take_stdin().expect("Could not take stdin");

    // Return stdin for the capture thread to write to
    // ...

    Ok(())
}
```

### Writing frames from windows-capture:

```rust
// In the GraphicsCaptureApiHandler::on_frame_arrived:
fn on_frame_arrived(
    &mut self,
    frame: &mut Frame,
    _capture_control: InternalCaptureControl,
) -> Result<(), Self::Error> {
    // Get raw BGRA buffer from the frame
    let buffer = frame.buffer()?;

    // Write directly to FFmpeg's stdin
    self.ffmpeg_stdin.write_all(&buffer)?;

    Ok(())
}
```

### With audio (desktop + mic):
For audio, we have two options:
1. **Separate process**: Capture audio with WASAPI, pipe to a second FFmpeg, mux later
2. **Named pipe**: Send audio to FFmpeg via a named pipe while video goes through stdin

Option 1 is simpler for v1:
```
# Video encoding (NVENC)
ffmpeg -f rawvideo ... -i pipe:0 -c:v h264_nvenc ... -y video_only.mp4

# Audio capture (WASAPI loopback via FFmpeg)
ffmpeg -f dshow -i audio="CABLE Output" -c:a aac -b:a 192k -y audio.aac

# Final mux (fast, no re-encoding)
ffmpeg -i video_only.mp4 -i audio.aac -c copy -y final.mp4
```

---

## Ring Buffer Strategy with FFmpeg Sidecar

For the replay buffer, we DON'T pipe frames to FFmpeg in real-time.
Instead:
1. **Capture**: windows-capture delivers BGRA frames
2. **Encode**: We pipe frames to FFmpeg in real-time → encoded H.264 packets
3. **Buffer**: Store encoded packets (NAL units) in a circular ring buffer in RAM
4. **Save trigger**: On hotkey, drain the ring buffer contents into an MP4 container

Alternative simpler approach for v1:
1. **Capture**: windows-capture delivers BGRA frames
2. **Buffer**: Store raw BGRA frames in a circular buffer (higher RAM, simpler)
3. **Save trigger**: On hotkey, pipe buffered frames to FFmpeg → MP4

The raw frame buffer approach uses more RAM but is much simpler:
- 1080p60, 30s buffer: 1920×1080×4 bytes × 60fps × 30s = ~14 GB (too much!)
- Need encoded buffer approach for production

For v1 prototype, we can use a shorter buffer or encoded approach.

---

## Bundling FFmpeg with Tauri

For distribution, bundle ffmpeg.exe as a Tauri sidecar:

1. Place `ffmpeg-x86_64-pc-windows-msvc.exe` in `src-tauri/binaries/`
2. Configure in `tauri.conf.json`:
```json
{
  "bundle": {
    "externalBin": ["binaries/ffmpeg"]
  }
}
```
3. Access from Rust:
```rust
use tauri::Manager;
let sidecar_path = app.path().resolve("binaries/ffmpeg.exe", BaseDirectory::Resource)?;
```

For dev, just use the system FFmpeg that's already in PATH.

---

## Fallback: Software Encoding
If NVENC is unavailable (no NVIDIA GPU), fall back to:
```
-c:v libx264 -crf 20 -preset fast -profile:v high
```
Note: CRF (not CQP) for x264. CRF 20 ≈ CQP 20 in visual quality.
