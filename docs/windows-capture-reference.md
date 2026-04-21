# windows-capture Crate — Quick Reference

**Crate**: `windows-capture` v2.0.0
**Source**: https://github.com/NiiightmareXD/windows-capture
**Docs**: https://docs.rs/windows-capture

## Overview
Uses Windows Graphics Capture API (WGC) for screen capture. Also supports DXGI Desktop Duplication.
Built-in hardware-accelerated video encoder with audio support.

## Installation
```toml
[dependencies]
windows-capture = "2.0.0"
```

## Key Types

| Type | Purpose |
|---|---|
| `GraphicsCaptureApiHandler` | Trait to implement for capture callbacks |
| `Settings` | Capture configuration (target, cursor, color format, etc.) |
| `Frame` | Captured frame — get buffer, save as image, send to encoder |
| `VideoEncoder` | Built-in H.264/HEVC encoder (hardware-accelerated) |
| `VideoSettingsBuilder` | Configure encoder width, height, bitrate, framerate |
| `AudioSettingsBuilder` | Configure audio capture settings |
| `ContainerSettingsBuilder` | Configure output container (MP4) |
| `Monitor` | Enumerate/select monitors |
| `Window` | Enumerate/select windows |
| `GraphicsCapturePicker` | System dialog to pick capture target |
| `InternalCaptureControl` | Stop capture from within handler |

## Basic Capture Pattern

```rust
use windows_capture::capture::{Context, GraphicsCaptureApiHandler};
use windows_capture::encoder::{
    AudioSettingsBuilder, ContainerSettingsBuilder, VideoEncoder, VideoSettingsBuilder,
};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::settings::*;

struct Capture {
    encoder: Option<VideoEncoder>,
    start: std::time::Instant,
}

impl GraphicsCaptureApiHandler for Capture {
    type Flags = (i32, i32);  // width, height
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn new(ctx: Context<Self::Flags>) -> Result<Self, Self::Error> {
        let encoder = VideoEncoder::new(
            VideoSettingsBuilder::new(ctx.flags.0 as u32, ctx.flags.1 as u32),
            AudioSettingsBuilder::default().disabled(true),
            ContainerSettingsBuilder::default(),
            "output.mp4",
        )?;

        Ok(Self {
            encoder: Some(encoder),
            start: std::time::Instant::now(),
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        // Send frame to encoder
        self.encoder.as_mut().unwrap().send_frame(frame)?;

        // Stop after N seconds
        if self.start.elapsed().as_secs() >= 30 {
            self.encoder.take().unwrap().finish()?;
            capture_control.stop();
        }

        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), Self::Error> {
        println!("Capture ended");
        Ok(())
    }
}
```

## Starting Capture

```rust
// Pick target via system dialog
let item = GraphicsCapturePicker::pick_item()
    .expect("Failed to pick item");

// Or programmatically select primary monitor
use windows_capture::monitor::Monitor;
let monitor = Monitor::primary()?;

// Or enumerate all monitors
let monitors = Monitor::enumerate()?;

// Or find a specific window
use windows_capture::window::Window;
let windows = Window::enumerate()?;
let game_window = windows.iter().find(|w| {
    w.title().map(|t| t.contains("Game")).unwrap_or(false)
});
```

## Settings Configuration

```rust
let settings = Settings::new(
    item,                                        // capture target
    CursorCaptureSettings::Default,              // capture cursor
    DrawBorderSettings::Default,                 // draw border around captured area
    SecondaryWindowSettings::Default,            // include secondary windows
    MinimumUpdateIntervalSettings::Default,      // frame rate limit (default 60fps)
    DirtyRegionSettings::Default,                // dirty region tracking
    ColorFormat::Rgba8,                          // pixel format
    (width, height),                             // flags passed to handler
);

// Start capture (blocks current thread)
Capture::start(settings).expect("Capture failed");
```

## Frame Rate Control
- Default: 60 FPS (16.67ms minimum interval)
- Custom: Use `MinimumUpdateIntervalSettings::Custom(Duration::from_millis(8))` for ~120 FPS
- The interval is a minimum — frames won't arrive faster than this

## Stream-Based Encoding
Instead of writing to a file, write to an in-memory stream:
```rust
use windows::Storage::Streams::InMemoryRandomAccessStream;
use windows::core::Interface;

let stream = InMemoryRandomAccessStream::new()?;
let encoder = VideoEncoder::new_from_stream(
    VideoSettingsBuilder::new(width, height),
    AudioSettingsBuilder::default().disabled(true),
    ContainerSettingsBuilder::default(),
    stream.cast()?,
)?;
```

## VideoSettingsBuilder Options
```rust
VideoSettingsBuilder::new(width, height)
    // Additional settings available via builder methods
    // Check docs.rs for full API
```

## DXGI Desktop Duplication (Alternative)
```rust
use windows_capture::dxgi_duplication_api::DxgiDuplicationApi;
use windows_capture::monitor::Monitor;

let monitor = Monitor::primary()?;
let mut dup = DxgiDuplicationApi::new(monitor)?;
let mut frame = dup.acquire_next_frame(33)?;  // timeout in ms
frame.save_as_image("screenshot.png", ImageFormat::Png)?;
```

## Important Notes
- `Capture::start()` **blocks the current thread** — run in a separate thread for Tauri integration
- The built-in `VideoEncoder` uses Windows Media Foundation (hardware-accelerated)
- For our use case, we'll use the built-in encoder instead of `ffmpeg-next` since `windows-capture` v2.0 has its own encoder
- Frame buffer is BGRA/RGBA format depending on `ColorFormat` setting
- The crate handles GPU → CPU data transfer internally
