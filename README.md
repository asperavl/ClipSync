# ClipSync

**Privacy-first, always-on gameplay clip recorder for Windows.**

ClipSync runs a silent, hardware-accelerated replay buffer in the background. When something epic happens, press **Ctrl+F9** to instantly save the last 30 seconds — no manual recording needed, no re-encoding, no lag.

## How It Works

```
                  always running
                       │
     ┌─────────────────▼──────────────────┐
     │  Screen Capture (WGC)              │
     │  ► Tick-based 60fps (like OBS)     │
     │  ► BGRA frames → FFmpeg stdin      │
     ├────────────────────────────────────┤
     │  Audio Capture (WASAPI Loopback)   │
     │  ► Captures desktop audio          │
     │  ► Raw PCM → FFmpeg named pipe     │
     ├────────────────────────────────────┤
     │  FFmpeg Encoder                    │
     │  ► NVENC H.264 CQP 20             │
     │  ► Segment muxer (2s chunks)       │
     │  ► Rolling buffer in %TEMP%        │
     └─────────────────┬──────────────────┘
                       │
              user presses Ctrl+F9
                       │
     ┌─────────────────▼──────────────────┐
     │  Clip Extraction                   │
     │  ► Concat last 15 segments         │
     │  ► Zero re-encoding (ffmpeg copy)  │
     │  ► Saved to ~/Videos/ClipSync/     │
     │  ► Takes < 1 second                │
     └───────────────────────────────────┘
```

## Features

- **Always-on replay buffer** — never miss a moment
- **Instant clip saving** — Ctrl+F9 saves the last 30 seconds with zero re-encoding
- **Hardware-accelerated** — NVENC encoding with automatic libx264 fallback
- **Desktop audio capture** — WASAPI loopback records whatever you hear (no virtual cables needed)
- **Tick-based frame timing** — OBS-style architecture for perfect 1:1 playback speed
- **Minimal footprint** — rolling 2-second segments, old ones auto-deleted
- **System tray** — runs silently with tray icon and right-click menu
- **Privacy-first** — everything stays local on your machine

## Requirements

- **Windows 10/11** (64-bit)
- **FFmpeg** in your PATH ([download](https://www.gyan.dev/ffmpeg/builds/))
- **NVIDIA GPU** recommended (for NVENC hardware encoding)
  - AMD/Intel GPUs work too — falls back to libx264 software encoding

## Getting Started

### Prerequisites

- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://rustup.rs/) (stable)
- [FFmpeg](https://www.gyan.dev/ffmpeg/builds/) (add to PATH)

### Build & Run

```bash
# Clone the repo
git clone https://github.com/YOUR_USERNAME/clipsync.git
cd clipsync

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

### First Launch

1. ClipSync starts and immediately begins buffering in the background
2. You'll see a system tray icon — right-click for options
3. **Disable the yellow screen border**: Go to Windows Settings → Privacy & security → Screenshot borders → Enable "Let apps turn off the screenshot border"
4. Play your game in **borderless fullscreen** mode
5. When something cool happens, press **Ctrl+F9**
6. Your clip is saved to `~/Videos/ClipSync/`

## Architecture

| Module | File | Purpose |
|---|---|---|
| **Capture** | `src-tauri/src/capture.rs` | WGC screen capture with tick-based frame timing |
| **Audio** | `src-tauri/src/audio.rs` | WASAPI loopback desktop audio capture |
| **Encoder** | `src-tauri/src/encoder.rs` | FFmpeg NVENC segment encoding |
| **Buffer** | `src-tauri/src/replay_buffer.rs` | Rolling segment management & clip extraction |
| **App** | `src-tauri/src/lib.rs` | Tauri app, tray, hotkey, state orchestration |

### Key Design Decisions

- **Tick-based capture** (not callback-based): A dedicated thread reads the latest frame at exactly 60fps intervals, regardless of monitor refresh rate. This guarantees correct playback speed — the same approach OBS uses.

- **Segment muxer**: FFmpeg writes 2-second MP4 segments with GOP-aligned keyframes. Clip saving concatenates them with `ffmpeg -c copy` — instant, lossless, no re-encoding.

- **WASAPI loopback**: Captures audio directly from the Windows audio engine — records whatever your speakers/headphones play. No virtual audio cables or Stereo Mix required.

- **Named pipe for audio**: Audio data flows from WASAPI → Windows named pipe → FFmpeg. This keeps audio and video in a single FFmpeg process for perfect A/V sync.

## Configuration

Currently hardcoded (settings UI coming soon):

| Setting | Default | Location |
|---|---|---|
| Buffer duration | 30 seconds | `lib.rs` → `BUFFER_DURATION_SECS` |
| Segment duration | 2 seconds | `lib.rs` → `SEGMENT_DURATION_SECS` |
| Resolution | 1920×1080 | `lib.rs` → `CAPTURE_WIDTH/HEIGHT` |
| Frame rate | 60 fps | `lib.rs` → `CAPTURE_FPS` |
| Quality (QP) | 20 | `lib.rs` → `ENCODE_QP` |
| Hotkey | Ctrl+F9 | `lib.rs` → setup |
| Output directory | ~/Videos/ClipSync/ | `lib.rs` → `get_output_dir()` |

## Known Limitations

- **Exclusive fullscreen not supported** — uses WGC which requires the Windows compositor. Use borderless fullscreen in your game settings.
- **No microphone capture yet** — only desktop audio is recorded.
- **No settings UI yet** — configuration requires code changes.

## Roadmap

- [ ] Settings UI (buffer duration, quality, hotkey, audio device selection)
- [ ] Google Drive upload with shareable link
- [ ] Clipboard integration (auto-copy share link)
- [ ] Microphone audio (separate track)
- [ ] Toast notifications on clip save
- [ ] Per-application audio capture

## Tech Stack

- **[Tauri v2](https://tauri.app/)** — lightweight native app framework
- **[windows-capture](https://crates.io/crates/windows-capture)** — Windows Graphics Capture API bindings
- **[wasapi](https://crates.io/crates/wasapi)** — Windows Audio Session API bindings
- **[ffmpeg-sidecar](https://crates.io/crates/ffmpeg-sidecar)** — FFmpeg subprocess management
- **React + TypeScript** — frontend (settings UI, planned)
- **Vite** — frontend build tool

## License

MIT
