# ClipSync — Development Updates

## [0.1.0] — 2026-04-21

### Initial Implementation

---

### 🎥 Screen Capture
- Integrated **Windows Graphics Capture (WGC)** API via `windows-capture` crate
- Primary monitor capture (1920×1080)
- **Tick-based frame timing** — a dedicated ticker thread reads the latest frame at exactly 60fps intervals, regardless of monitor refresh rate. This is the same architecture OBS uses and ensures 1:1 playback speed.
- Frame throttling drops excess frames from high-refresh monitors (e.g. 144Hz, 165Hz) — only 60fps gets sent to FFmpeg
- Graceful stop signal support

### 🔊 Desktop Audio Capture
- Integrated **WASAPI loopback** via `wasapi` crate
- Captures audio from the **default Windows output device** (speakers/headphones) — whatever the user hears gets recorded
- **Silence Injection** - Implemented automatic silence injection when WASAPI loopback drops out (e.g., when no audio is playing). This prevents FFmpeg from hanging/stalling due to audio pipe starvation, ensuring rock-solid encoding.
- Raw PCM f32le samples at 48kHz stereo

### ⚙️ FFmpeg Encoding
- **NVENC H.264** hardware encoding (CQP 20, high profile, B-frames disabled)
- Automatic fallback to **libx264** software encoding on non-NVIDIA systems
- **Segment muxer** — outputs rolling 2-second MP4 segments with GOP-aligned keyframes
- Video input via stdin (rawvideo BGRA), audio input via **Windows named pipe**
- Single FFmpeg process handles both audio and video for guaranteed A/V sync

### 📼 Replay Buffer & Storage
- **Always-on** — buffer starts automatically on app launch, no user interaction needed
- Rolling 30-second buffer of 2-second segments stored in `%TEMP%\clipsync_buffer`
- **Zero re-encoding clip extraction** — `ffmpeg -f concat -c copy` remuxes the last 15 segments in under 1 second
- Output videos saved strictly to `~/Videos/ClipSync/` to keep user folders clean.
- Automatically generates thumbnails (`.jpg`) and extracts duration metadata.
- **Frictionless AppData Migration** - All thumbnails and JSON metadata are securely stored in `%LOCALAPPDATA%\ClipSync\Metadata\`, completely hidden from the user's main video directory.
- **Self-Healing Clip Store** - Database detects orphaned clips (e.g., if a user deletes an `.mp4` manually) and cleans up its own database entries dynamically.

### 💻 UI & Frontend
- Fully responsive **React + TailwindCSS** frontend with a custom "Obsidian Neon" aesthetic.
- **Custom Title Bar** - Frameless window layout with custom drag regions and custom minimize/maximize/hide buttons.
- Real-time backend data binding via Tauri `invoke`.
- Built-in `convertFileSrc` asset protocol usage to securely serve local `.jpg` and `.mp4` files from the filesystem straight to the React webview.
- **Dynamic Clip Library** - Renders thumbnails, relative timestamps, and clip duration.
- **Clip Detail Player** - Built-in video player dynamically loads the clicked clip.

### ⌨️ Global Hotkey & System Tray
- **Ctrl+F9** registered as a global shortcut via `tauri-plugin-global-shortcut`
- Tray icon with right-click menu: **Save Clip**, **Pause Buffer**, **Settings**, **Quit**
- Left-click tray icon → opens main window
- Tooltip shows current status

### 🐛 Bugs Fixed During Development
- **FFmpeg Pipe Stalls** - WASAPI loopback stops sending bytes when silent, causing the FFmpeg pipe to block indefinitely. Fixed by injecting exact lengths of silence blocks based on `Instant::now()` elapsed duration.
- **Slow-motion clips** — WGC delivers frames at monitor refresh rate; fixed by tick-based design that enforces exact 60fps to FFmpeg
- **Sped-up clips** — opposite problem after overcorrection; resolved by accurate interval timing with `Instant::now()`
- **`moov atom not found`** on concat — FFmpeg is still writing the latest segment when save is triggered; fixed by always skipping the last segment
- **Tauri Title Bar Drag Interception** - Custom title bar buttons were unclickable because `data-tauri-drag-region` swallowed pointer events. Fixed by using layout isolation rather than overlapping regions.

---

## Upcoming

- [ ] Settings UI Data Binding (buffer duration, quality, hotkey, audio device)
- [ ] Google Drive upload with shareable link
- [ ] Auto-copy share link to clipboard
- [ ] Clip Actions (Delete/Favorite)
- [ ] Microphone capture (separate audio track)
- [ ] Toast notification on clip save
