mod audio;
mod capture;
mod encoder;
mod replay_buffer;
mod clip_store;

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tauri::{
    Manager,
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
};

use crate::audio::AudioConfig;
use crate::capture::CaptureConfig;
use crate::encoder::{Encoder, EncoderConfig};
use crate::replay_buffer::ReplayBuffer;

// ── Constants ──────────────────────────────────────────────

const BUFFER_DURATION_SECS: u32 = 30; // Save last 30 seconds
const SEGMENT_DURATION_SECS: u32 = 2; // Each segment is 2 seconds
const CAPTURE_WIDTH: u32 = 1920;
const CAPTURE_HEIGHT: u32 = 1080;
const CAPTURE_FPS: u32 = 60;
const ENCODE_QP: u32 = 20;

// ── App State ──────────────────────────────────────────────

/// Shared app state accessible from commands and event handlers.
struct AppState {
    /// Whether the replay buffer is running
    is_buffering: AtomicBool,
    /// Signal to stop the capture loop
    stop_signal: Arc<AtomicBool>,
    /// Total frames captured since app start
    frame_count: Arc<AtomicU64>,
    /// Handle to the capture thread
    capture_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
    /// Handle to the audio capture thread
    audio_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
    /// Handle to the encoder process
    encoder_handle: Mutex<Option<Encoder>>,
    /// Replay buffer manager (segment cleanup + clip extraction)
    replay_buffer: ReplayBuffer,
    /// Lock to prevent concurrent save operations
    saving: AtomicBool,
    /// Store for managing clip metadata
    clip_store: clip_store::ClipStore,
}

// ── Replay Buffer Control ──────────────────────────────────

/// Start the always-on replay buffer (capture + encode to segments).
fn start_buffer(state: &AppState) {
    if state.is_buffering.load(Ordering::Relaxed) {
        println!("[ClipSync] Buffer already running");
        return;
    }

    println!("[ClipSync] Starting replay buffer...");

    // Reset state
    state.stop_signal.store(false, Ordering::Relaxed);
    state.frame_count.store(0, Ordering::Relaxed);

    // Clean any leftover segments from previous session
    state.replay_buffer.clear();

    // Audio config
    let audio_config = AudioConfig::default();

    // Configure encoder with segment output + audio
    let encoder_config = EncoderConfig {
        width: CAPTURE_WIDTH,
        height: CAPTURE_HEIGHT,
        fps: CAPTURE_FPS,
        qp: ENCODE_QP,
        segment_pattern: state.replay_buffer.segment_pattern(),
        segment_duration: SEGMENT_DURATION_SECS,
        enable_audio: true,
        audio_sample_rate: audio_config.sample_rate,
        audio_channels: audio_config.channels,
    };

    // Step 1: Start audio capture thread (creates named pipe, waits for FFmpeg)
    let stop_audio = state.stop_signal.clone();
    let audio_handle = std::thread::Builder::new()
        .name("clipsync-audio".into())
        .spawn(move || {
            if let Err(e) = audio::start_audio_capture(&audio_config, stop_audio) {
                eprintln!("[ClipSync] Audio capture error: {}", e);
            }
        })
        .expect("Failed to spawn audio thread");
    *state.audio_thread.lock().unwrap() = Some(audio_handle);

    // Brief delay to let the named pipe be created before FFmpeg tries to open it
    std::thread::sleep(std::time::Duration::from_millis(200));

    // Step 2: Start FFmpeg (connects to audio named pipe + takes video from stdin)
    let (encoder, ffmpeg_stdin) = match Encoder::start(&encoder_config) {
        Ok(result) => result,
        Err(e) => {
            eprintln!("[ClipSync] Failed to start encoder: {}", e);
            state.stop_signal.store(true, Ordering::Relaxed);
            return;
        }
    };

    *state.encoder_handle.lock().unwrap() = Some(encoder);

    // Configure capture
    let capture_config = CaptureConfig {
        width: CAPTURE_WIDTH,
        height: CAPTURE_HEIGHT,
        fps: CAPTURE_FPS,
    };

    let stop_signal = state.stop_signal.clone();
    let frame_count = state.frame_count.clone();

    // Spawn capture on a dedicated thread (blocks)
    let handle = std::thread::Builder::new()
        .name("clipsync-capture".into())
        .spawn(move || {
            if let Err(e) =
                capture::start_capture(&capture_config, ffmpeg_stdin, stop_signal, frame_count)
            {
                eprintln!("[ClipSync] Capture error: {}", e);
            }
        })
        .expect("Failed to spawn capture thread");

    *state.capture_thread.lock().unwrap() = Some(handle);
    state.is_buffering.store(true, Ordering::Relaxed);

    // Start segment cleanup thread
    let buffer_dir = state.replay_buffer.buffer_dir().to_path_buf();
    let max_segs = (BUFFER_DURATION_SECS / SEGMENT_DURATION_SECS) as usize + 2;
    let stop_cleanup = state.stop_signal.clone();
    std::thread::Builder::new()
        .name("clipsync-cleanup".into())
        .spawn(move || {
            while !stop_cleanup.load(Ordering::Relaxed) {
                // Clean old segments every second
                cleanup_segments(&buffer_dir, max_segs);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        })
        .expect("Failed to spawn cleanup thread");

    println!("[ClipSync] Replay buffer started ({}s @ {}fps)", BUFFER_DURATION_SECS, CAPTURE_FPS);
}

/// Clean old segments, keeping only the most recent N.
fn cleanup_segments(buffer_dir: &std::path::Path, max_segments: usize) {
    let mut segments: Vec<std::path::PathBuf> = std::fs::read_dir(buffer_dir)
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().map(|ext| ext == "mp4").unwrap_or(false)
                && p.file_name()
                    .map(|n| n.to_string_lossy().starts_with("seg_"))
                    .unwrap_or(false)
        })
        .collect();

    segments.sort();

    if segments.len() > max_segments {
        let to_remove = segments.len() - max_segments;
        for seg in segments.iter().take(to_remove) {
            let _ = std::fs::remove_file(seg);
        }
    }
}

/// Stop the replay buffer.
fn stop_buffer(state: &AppState) {
    if !state.is_buffering.load(Ordering::Relaxed) {
        return;
    }

    println!("[ClipSync] Stopping replay buffer...");

    state.stop_signal.store(true, Ordering::Relaxed);

    // Wait for capture thread
    if let Some(handle) = state.capture_thread.lock().unwrap().take() {
        let _ = handle.join();
    }

    // Wait for audio thread
    if let Some(handle) = state.audio_thread.lock().unwrap().take() {
        let _ = handle.join();
    }

    // Finish encoder
    if let Some(encoder) = state.encoder_handle.lock().unwrap().take() {
        if let Err(e) = encoder.finish() {
            eprintln!("[ClipSync] Encoder finish error: {}", e);
        }
    }

    let frames = state.frame_count.load(Ordering::Relaxed);
    state.is_buffering.store(false, Ordering::Relaxed);
    println!("[ClipSync] Replay buffer stopped ({} total frames)", frames);
}

/// Save a clip from the replay buffer (triggered by hotkey).
fn save_clip(state: &AppState) {
    // Prevent concurrent saves
    if state
        .saving
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        println!("[ClipSync] Already saving a clip, please wait...");
        return;
    }

    if !state.is_buffering.load(Ordering::Relaxed) {
        println!("[ClipSync] Buffer not running, cannot save clip");
        state.saving.store(false, Ordering::Relaxed);
        return;
    }

    println!("[ClipSync] ═══════════════════════════════════════");
    println!("[ClipSync] SAVING CLIP (last {}s)...", BUFFER_DURATION_SECS);
    println!("[ClipSync] ═══════════════════════════════════════");

    match state.replay_buffer.save_clip() {
        Ok(path) => {
            println!("[ClipSync] ✓ Clip saved: {}", path.display());
            
            let id = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs().to_string();
            let date_recorded = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs();
            
            let thumb = state.clip_store.generate_thumbnail(&path, &id).unwrap_or_default();
            
            let meta = clip_store::ClipMetadata {
                id: id.clone(),
                title: format!("Clip {}", id),
                game_name: None,
                date_recorded,
                duration_secs: BUFFER_DURATION_SECS,
                cloud_status: "local".to_string(),
                is_favorite: false,
                file_path: path.to_string_lossy().into_owned(),
                thumbnail_path: thumb.to_string_lossy().into_owned(),
            };
            
            if let Err(e) = state.clip_store.add_clip(meta) {
                eprintln!("[ClipSync] Failed to add clip metadata: {}", e);
            }
        }
        Err(e) => {
            eprintln!("[ClipSync] ✗ Failed to save clip: {}", e);
        }
    }

    state.saving.store(false, Ordering::Relaxed);
}

// ── Tauri Commands ──────────────────────────────────────────

#[tauri::command]
fn get_status(state: tauri::State<'_, AppState>) -> String {
    if state.saving.load(Ordering::Relaxed) {
        "saving".to_string()
    } else if state.is_buffering.load(Ordering::Relaxed) {
        "buffering".to_string()
    } else {
        "idle".to_string()
    }
}

#[tauri::command]
fn trigger_save_clip(state: tauri::State<'_, AppState>) -> String {
    save_clip(&state);
    "save_triggered".to_string()
}

#[tauri::command]
fn get_frame_count(state: tauri::State<'_, AppState>) -> u64 {
    state.frame_count.load(Ordering::Relaxed)
}

#[tauri::command]
fn get_all_clips(state: tauri::State<'_, AppState>) -> Vec<clip_store::ClipMetadata> {
    state.clip_store.get_clips()
}

#[tauri::command]
fn update_clip_meta(state: tauri::State<'_, AppState>, clip: clip_store::ClipMetadata) -> Result<(), String> {
    state.clip_store.update_clip(clip).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_clip_meta(state: tauri::State<'_, AppState>, id: String) -> Result<(), String> {
    state.clip_store.delete_clip(&id).map_err(|e| e.to_string())
}

// ── App Entry ──────────────────────────────────────────────

fn get_output_dir() -> std::path::PathBuf {
    let videos_dir = dirs_next::video_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    videos_dir.join("ClipSync")
}

fn get_app_data_dir() -> std::path::PathBuf {
    let data_dir = dirs_next::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    data_dir.join("ClipSync").join("Metadata")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState {
            is_buffering: AtomicBool::new(false),
            stop_signal: Arc::new(AtomicBool::new(false)),
            frame_count: Arc::new(AtomicU64::new(0)),
            capture_thread: Mutex::new(None),
            audio_thread: Mutex::new(None),
            encoder_handle: Mutex::new(None),
            replay_buffer: ReplayBuffer::new(
                BUFFER_DURATION_SECS,
                SEGMENT_DURATION_SECS,
                get_output_dir(),
            ),
            saving: AtomicBool::new(false),
            clip_store: clip_store::ClipStore::new(get_app_data_dir()),
        })
        .setup(|app| {
            // ── System Tray ──────────────────────────────────────
            let save_clip_i =
                MenuItem::with_id(app, "save_clip", "Save Clip (Ctrl+F9)", true, None::<&str>)?;
            let pause_i =
                MenuItem::with_id(app, "pause", "Pause Buffer", true, None::<&str>)?;
            let settings_i =
                MenuItem::with_id(app, "settings", "Settings", true, None::<&str>)?;
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

            let menu =
                Menu::with_items(app, &[&save_clip_i, &pause_i, &settings_i, &quit_i])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .tooltip("ClipSync — Buffering")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "save_clip" => {
                        let state = app.state::<AppState>();
                        save_clip(&state);
                    }
                    "pause" => {
                        let state = app.state::<AppState>();
                        if state.is_buffering.load(Ordering::Relaxed) {
                            stop_buffer(&state);
                        } else {
                            start_buffer(&state);
                        }
                    }
                    "settings" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "quit" => {
                        let state = app.state::<AppState>();
                        stop_buffer(&state);
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.unminimize();
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            // ── Global Shortcut (Ctrl+F9 = Save Clip) ────────────
            #[cfg(desktop)]
            {
                use tauri_plugin_global_shortcut::{
                    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState,
                };

                let save_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::F9);
                let app_handle = app.handle().clone();

                app.handle().plugin(
                    tauri_plugin_global_shortcut::Builder::new()
                        .with_handler(move |_app, shortcut, event| {
                            if shortcut == &save_shortcut {
                                if let ShortcutState::Pressed = event.state() {
                                    println!("[ClipSync] Ctrl+F9 pressed — saving clip!");
                                    let state = app_handle.state::<AppState>();
                                    save_clip(&state);
                                }
                            }
                        })
                        .build(),
                )?;

                app.global_shortcut().register(save_shortcut)?;
                println!("[ClipSync] Global shortcut Ctrl+F9 registered (Save Clip)");
            }

            // ── Auto-start replay buffer ─────────────────────────
            let app_handle = app.handle().clone();
            std::thread::spawn(move || {
                // Small delay to let the app fully initialize
                std::thread::sleep(std::time::Duration::from_millis(500));
                let state = app_handle.state::<AppState>();
                start_buffer(&state);
            });

            println!("[ClipSync] App started — replay buffer will begin shortly");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            trigger_save_clip,
            get_frame_count,
            get_all_clips,
            update_clip_meta,
            delete_clip_meta,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
