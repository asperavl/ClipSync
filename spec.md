# ClipSync — Product Specification

**Version**: 0.1 (Prototype)
**Last Updated**: 2026-04-21

---

## 1. Overview

ClipSync is a privacy-first gameplay clip recorder for Windows. Users press a hotkey to save the last N seconds of gameplay, which auto-uploads to their personal Google Drive and generates a shareable link. Clips are viewed through a custom web player in their original recorded quality — no re-encoding, no compression, no quality loss.

### Core Principle

> **Users own their clips.** Recordings are stored on the user's own Google Drive — not on third-party servers. ClipSync never claims any license, never uses clips for AI training, and never shares data with advertisers. The user has full control over their content at all times.

### What Makes ClipSync Different

| Feature | Medal.tv | Outplayed | ShadowPlay | OBS | ClipSync |
|---|---|---|---|---|---|
| Clip ownership | Medal's servers | Overwolf's AWS | Local only | Local only | **User's Google Drive** |
| Content license | Granted to Medal | Perpetual to Overwolf | None | None | **None** |
| AI training | Yes (opt-out) | Unknown | No | No | **Never** |
| Custom web viewer | Yes (their platform) | Yes (their platform) | No | No | **Yes (user-controlled)** |
| Zero re-compression | No | No | N/A | N/A | **Yes** |
| Cloud upload | To Medal | To Overwolf | No | No | **To user's Drive** |
| Share links | Yes | Yes | No | No | **Yes (custom viewer)** |
| GPU vendor support | All | All | NVIDIA only | All | **All (NVENC/AMF/QSV)** |
| Cost | Free (ad-supported) | Free (ad-supported) | Free | Free | **Free** |

---

## 2. Target Platform

- **OS**: Windows 10 (version 1809+) and Windows 11
- **GPU**: Any modern GPU with hardware encoding support
  - NVIDIA (NVENC): GTX 600 series+
  - AMD (AMF): RX 400 series+
  - Intel (QSV): 6th gen Core+
- **RAM**: 4 GB minimum, 8 GB+ recommended
- **Storage**: Sufficient disk space for temporary clip storage before upload
- **Network**: Required for Google Drive upload and share link generation

---

## 3. User Experience

### 3.1 First Launch

1. App installs and starts minimized to the system tray
2. Welcome window opens with setup wizard:
   - Select capture target (monitor or specific window)
   - Choose resolution and FPS
   - Set replay buffer duration
   - Configure hotkey
   - Connect Google Drive account (OAuth)
3. Setup complete → app runs in system tray

### 3.2 Core Loop

```
User plays a game
        ↓
ClipSync silently records in the background (replay buffer)
        ↓
Something cool happens
        ↓
User presses hotkey (default: Ctrl+F9)
        ↓
Last N seconds saved as MP4 to local disk
        ↓
Clip auto-uploads to Google Drive (background)
        ↓
Share link copied to clipboard
        ↓
Toast notification: "Clip saved! Link copied ✓"
        ↓
User pastes link in Discord / Twitter / wherever
        ↓
Viewers open link → custom web player → original quality playback
```

### 3.3 System Tray

The app lives in the system tray. Right-click menu:

- **Status**: Recording (green) / Paused (yellow) / Off (gray)
- **Save Clip** (same as hotkey)
- **Pause / Resume** recording
- **Open Clip Library**
- **Settings**
- **Quit**

### 3.4 Settings Window

| Category | Setting | Options | Default |
|---|---|---|---|
| **Capture** | Target | Monitor / Window | Primary monitor |
| | Resolution | 1080p / 1440p / 4K | Match display |
| | FPS | 30 / 60 / 120* | 60 |
| **Buffer** | Duration | 15s / 30s / 60s / 120s | 30s |
| **Audio** | Desktop audio | On / Off | On |
| | Microphone | On / Off | Off |
| | Mic device | [dropdown] | Default |
| **Hotkey** | Save clip | Key combo | Ctrl+F9 |
| **Upload** | Google Drive | Connected / Disconnected | — |
| | Auto-upload | On / Off | On |
| **Storage** | Local save folder | Path | ~/Videos/ClipSync |

*\* 120 FPS available only at 1080p and below.*

### 3.5 Resolution & FPS Matrix

| Resolution | 30 FPS | 60 FPS | 120 FPS |
|---|---|---|---|
| 1080p (1920×1080) | ✅ | ✅ | ✅ |
| 1440p (2560×1440) | ✅ | ✅ | ❌ |
| 4K (3840×2160) | ✅ | ✅ | ❌ |

---

## 4. Technical Specification

### 4.1 Encoding

| Parameter | Value | Notes |
|---|---|---|
| **Codec** | H.264 (AVC) | Most compatible for browser playback |
| **Encoder** | NVENC / AMF / QSV | Hardware-accelerated, auto-detected |
| **Rate control** | CQP (Constant QP) | Fixed quality, not user-configurable |
| **CQP value** | 20 | Visually excellent, good file sizes |
| **Profile** | High | Best compression efficiency |
| **Pixel format** | NV12 | Standard for hardware encoders |
| **Container** | MP4 | Best compatibility with Drive + browsers |
| **Audio codec** | AAC-LC | 192 kbps stereo for desktop + mic |
| **GOP size** | 2 seconds | Keyframe every 2s for clean buffer cuts |
| **B-frames** | 0 | Lower latency, simpler ring buffer |

### 4.2 File Size Estimates (CQP 20, 30-second clips)

| Resolution | FPS | Bitrate (approx.) | File Size |
|---|---|---|---|
| 1080p | 30 | ~10 Mbps | ~37 MB |
| 1080p | 60 | ~18 Mbps | ~68 MB |
| 1080p | 120 | ~30 Mbps | ~112 MB |
| 1440p | 60 | ~30 Mbps | ~112 MB |
| 4K | 60 | ~60 Mbps | ~225 MB |

### 4.3 RAM Usage (Ring Buffer)

| Resolution + FPS | 15s | 30s | 60s | 120s |
|---|---|---|---|---|
| 1080p @ 60 | 34 MB | 68 MB | 135 MB | 270 MB |
| 1080p @ 120 | 56 MB | 112 MB | 225 MB | 450 MB |
| 1440p @ 60 | 56 MB | 112 MB | 225 MB | 450 MB |
| 4K @ 60 | 112 MB | 225 MB | 450 MB | 900 MB |

### 4.4 Capture Pipeline

```
Screen (WGC API via windows-capture)
    ↓ BGRA frames
    ↓ pipe via stdin
FFmpeg subprocess (ffmpeg-sidecar)
    ↓ h264_nvenc -qp 20 -profile:v high -g {fps*2} -bf 0
    ↓ encoded H.264 packets → MP4
Ring Buffer (circular, N seconds, GOP-aligned encoded packets)
    ↓ [on hotkey trigger]
Flush to MP4 file
    ↓
Local file → Upload pipeline
```

#### FFmpeg Integration
- **Approach**: `ffmpeg-sidecar` crate — spawns FFmpeg as a subprocess
- **Why not C bindings**: `ffmpeg-next` requires complex C library linking on Windows. Sidecar gives identical NVENC quality with zero build complexity.
- **Frame pipeline**: BGRA frames from `windows-capture` → piped to FFmpeg stdin → NVENC encodes with CQP 20
- **Distribution**: FFmpeg binary bundled as a Tauri sidecar resource
- **Fallback**: If NVENC unavailable, auto-falls back to `libx264 -crf 20`

#### Screen Capture
- **API**: Windows Graphics Capture (WGC)
- **Fallback**: DXGI Desktop Duplication (for older Windows versions)
- **Fullscreen support**: WGC captures any game running through the Desktop Window Manager (DWM), which includes:
  - Borderless windowed (always works)
  - "Fullscreen" with Windows Fullscreen Optimizations enabled (default on Win10/11) — Windows silently converts these to borderless, so WGC captures them fine
  - DX12 games (most don't support true exclusive fullscreen at all)
  - This covers **95%+ of modern games** (Valorant, CS2, Apex, Fortnite, League, OW2, CoD, etc.)
- **Not supported (v1)**: True exclusive fullscreen with Fullscreen Optimizations manually disabled (rare). Requires game injection via DLL hooking, planned for v2.

#### Audio Capture
- **Desktop audio**: WASAPI loopback capture (captures system audio output)
- **Microphone**: WASAPI input capture
- Both streams are independently toggleable
- Mixed into the final MP4 as a single stereo AAC track
  - Future consideration: separate audio tracks for post-editing

#### Ring Buffer
- Stores encoded video and audio packets in RAM
- Fixed duration, configurable by user
- Organized as a queue of GOP-aligned chunks (each chunk starts with a keyframe)
- On save trigger: oldest chunks are discarded if beyond buffer duration, remaining chunks are drained and muxed into MP4
- Thread-safe: capture threads write, save trigger reads

### 4.5 Upload Pipeline

```
Local MP4 file
    ↓
Google Drive API v3 (resumable upload)
    ↓
Set permission: { type: "anyone", role: "reader", allowFileDiscovery: false }
    ↓
Register clip metadata to Cloudflare KV (via Worker API)
    ↓
Share URL: https://clipsync.pages.dev/c/{clipId}
    ↓
Copy to clipboard
```

#### Google Drive Integration
- **Auth**: OAuth 2.0 Desktop flow
- **Scope**: `drive.file` (app can only access files it created)
- **Upload**: Resumable upload (supports large files + network interruptions)
- **Permissions**: Unlisted — anyone with the link can view, not discoverable via search
- **Folder**: All clips stored in a "ClipSync" folder in user's Drive root

#### Clip Registration
- After upload, the desktop app calls the Cloudflare Worker `/register` endpoint
- Stores clip metadata in Cloudflare KV:
  ```json
  {
    "driveFileId": "1a2b3c4d5e...",
    "title": "clip_2026-04-21_14-30-22",
    "duration": 30,
    "resolution": "1920x1080",
    "fps": 60,
    "size": 71500000,
    "createdAt": "2026-04-21T14:30:22Z"
  }
  ```
- Clip ID is a short unique string (nanoid, 8 characters)

---

## 5. Web Viewer

### 5.1 Share Link Format

```
https://clipsync.pages.dev/c/{clipId}

Example: https://clipsync.pages.dev/c/x7kQ9m2p
```

### 5.2 Viewer Page

- **Hosting**: Cloudflare Pages (static site, free)
- **Video player**: Plyr.js (open-source, lightweight, customizable)
- **Video source**: Streams original MP4 from user's Google Drive via Cloudflare Worker proxy

#### Viewer Features
- Custom HTML5 video player (Plyr.js)
- Original quality playback — no transcoding, no adaptive bitrate
- Clip metadata: duration, resolution, FPS, date
- Download button (downloads the original MP4)
- Dark theme
- Responsive design (desktop + mobile)
- OG meta tags (rich previews in Discord, Twitter, etc.)
- Subtle "Recorded with ClipSync" branding

#### What the viewer does NOT do
- No account required to view
- No ads
- No tracking
- No comments section
- No social features
- No re-encoding or quality reduction

### 5.3 Cloudflare Worker (CORS Proxy)

Google Drive blocks direct video playback from custom domains (CORS policy). The Cloudflare Worker acts as a transparent proxy:

```
Viewer's browser
    ↓ GET /stream/{clipId} (with Range header)
Cloudflare Worker
    ↓ Look up Drive file ID from KV
    ↓ Fetch video bytes from Google Drive
    ↓ Add CORS headers (Access-Control-Allow-Origin)
    ↓ Forward Range headers for seeking
    ↓ Stream response body (no buffering)
Viewer's browser
    ↓ <video> element plays original MP4
```

#### Worker Endpoints

| Endpoint | Method | Auth | Purpose |
|---|---|---|---|
| `/meta/{clipId}` | GET | None | Return clip metadata from KV |
| `/stream/{clipId}` | GET | None | Proxy video from Google Drive with CORS |
| `/register` | POST | Secret key | Register new clip (called by desktop app) |
| `/delete/{clipId}` | DELETE | Secret key | Remove clip metadata from KV |

#### Limits (Free Tier)
- 100,000 Worker requests per day
- ~5,000–20,000 video views per day (each view ≈ 5-20 range requests)
- 100,000 KV reads per day
- 1,000 KV writes per day

---

## 6. Technology Stack

### Desktop App

| Component | Technology |
|---|---|
| App framework | Tauri v2 |
| Backend language | Rust |
| Frontend | React + TypeScript |
| Screen capture | `windows-capture` crate (WGC API) |
| Audio capture | `wasapi` crate (loopback) + `cpal` crate (mic) |
| Encoding | `ffmpeg-sidecar` crate (pipes frames to FFmpeg subprocess for H.264 NVENC/AMF/QSV) |
| Global hotkeys | `tauri-plugin-global-shortcut` |
| Google Drive API | `reqwest` + `oauth2` crates |
| Token storage | `keyring` crate (OS credential store) |
| Clip IDs | `nanoid` crate |

### Web Viewer

| Component | Technology |
|---|---|
| Hosting | Cloudflare Pages |
| Video player | Plyr.js |
| Proxy/API | Cloudflare Workers |
| Metadata store | Cloudflare KV |

### Infrastructure Cost

| Service | Free Tier | Cost |
|---|---|---|
| Google Drive | 15 GB (user's account) | $0 |
| Cloudflare Pages | Unlimited | $0 |
| Cloudflare Workers | 100K req/day | $0 |
| Cloudflare KV | 100K reads/day | $0 |
| **Total** | | **$0** |

---

## 7. Project Structure

```
clipsync/
├── src/                           # React frontend (Tauri UI)
│   ├── App.tsx
│   ├── App.css
│   ├── main.tsx
│   └── components/
│       ├── StatusIndicator.tsx
│       ├── Settings.tsx
│       ├── ClipLibrary.tsx
│       └── DriveConnect.tsx
│
├── src-tauri/                     # Rust backend
│   ├── src/
│   │   ├── lib.rs                 # Tauri app entry + commands
│   │   ├── capture.rs             # Screen capture (WGC)
│   │   ├── encoder.rs             # H.264 encoding (NVENC/AMF/QSV)
│   │   ├── ring_buffer.rs         # Circular buffer for replay
│   │   ├── audio.rs               # Desktop + mic audio capture
│   │   ├── drive.rs               # Google Drive API client
│   │   ├── auth.rs                # OAuth 2.0 token management
│   │   └── clip.rs                # Clip metadata + ID generation
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── icons/
│   └── capabilities/
│       └── default.json
│
├── viewer/                        # Web viewer (Cloudflare Pages)
│   ├── index.html                 # Landing page
│   ├── clip.html                  # Clip viewer page
│   ├── css/
│   │   └── viewer.css
│   ├── js/
│   │   ├── player.js
│   │   └── clip-loader.js
│   └── assets/
│       └── logo.svg
│
├── worker/                        # Cloudflare Worker (CORS proxy)
│   ├── src/
│   │   └── index.js
│   ├── wrangler.toml
│   └── package.json
│
├── spec.md                        # This file
├── package.json
└── vite.config.ts
```

---

## 8. Security & Privacy

### Data Flow
- **Clips are stored ONLY on the user's Google Drive**
- The Cloudflare Worker proxy streams video on-demand but never stores it
- Cloudflare KV stores lightweight metadata only (clip ID, Drive file ID, duration, etc.)
- No telemetry, no analytics, no tracking
- No user accounts on ClipSync infrastructure

### Authentication
- **Google OAuth**: Standard OAuth 2.0 desktop flow. Refresh tokens stored in OS credential store (Windows Credential Manager)
- **Worker API**: `/register` and `/delete` endpoints protected by a secret API key (stored in Worker environment variables)
- **Viewer**: No authentication required. Anyone with the link can view (unlisted model, same as YouTube unlisted videos)

### What ClipSync NEVER does
- ❌ Train AI on user clips
- ❌ Claim any license over user content
- ❌ Share user data with third parties
- ❌ Show ads
- ❌ Track viewing behavior
- ❌ Store clips on ClipSync infrastructure
- ❌ Re-encode or compress clips

---

## 9. Limitations (v1)

| Limitation | Reason | Planned Resolution |
|---|---|---|
| No true exclusive fullscreen capture | WGC relies on DWM; rare edge case since Windows defaults to fullscreen optimizations | Game injection in v2 for the ~5% of cases |
| 120 FPS only at ≤1080p | Hardware encoder throughput | Future GPU gen support |
| Google Drive only | Scoped for v1 | OneDrive, Dropbox, S3 in v2 |
| No clip trimming | Prototype scope | Built-in trimmer in v2 |
| No game auto-detection | Prototype scope | Auto-start on game launch in v2 |
| Worker free tier limits | ~5K-20K views/day | Paid plan ($5/mo) if needed |
| Google Drive rate limiting | Not a CDN | Upgrade path to R2/CDN if viral |
| Single audio track | Simpler muxing | Multi-track audio in v2 |

---

## 10. Future Roadmap

### v2
- Clip trimmer (trim start/end before upload)
- Game auto-detection (start recording when game launches)
- Additional cloud providers (OneDrive, Dropbox, S3)
- Clip library dashboard
- Overlay notification on clip save
- Multi-track audio (separate desktop + mic)
- Webcam overlay

### v3
- Game capture via DLL injection (exclusive fullscreen)
- AI highlight detection (auto-clip based on game events)
- Custom domain support for viewer (bring your own domain)
- Discord bot (auto-post clips to a channel)
- Self-hostable viewer option
