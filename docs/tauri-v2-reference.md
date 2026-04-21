# Tauri v2 — Quick Reference

## Project Creation (PowerShell)

```powershell
# Interactive setup (recommended for initial scaffold)
irm https://create.tauri.app/ps | iex

# Or via npm
npm create tauri-app@latest
```

Choices for ClipSync:
- **Project name**: clipsync
- **Identifier**: com.clipsync.app
- **Language**: TypeScript / JavaScript
- **Package manager**: npm
- **UI template**: React
- **UI flavor**: TypeScript

After creation:
```powershell
cd clipsync
npm install
npm run tauri dev
```

---

## Configuration Files

### `src-tauri/tauri.conf.json`
Main config file. Key fields:
```json
{
  "productName": "ClipSync",
  "version": "0.1.0",
  "identifier": "com.clipsync.app",
  "build": {
    "frontendDist": "../dist",
    "devUrl": "http://localhost:1420",
    "beforeDevCommand": "npm run dev",
    "beforeBuildCommand": "npm run build"
  },
  "app": {
    "windows": [
      {
        "title": "ClipSync",
        "width": 900,
        "height": 600,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "trayIcon": {
      "iconPath": "icons/icon.png",
      "iconAsTemplate": true
    }
  }
}
```

### `src-tauri/Cargo.toml`
Rust dependencies. Add features as needed:
```toml
[dependencies]
tauri = { version = "2", features = ["tray-icon"] }
```

### `src-tauri/capabilities/default.json`
Permissions for the frontend:
```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "main-capability",
  "description": "Main window capability",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "global-shortcut:allow-is-registered"
  ]
}
```

---

## Calling Rust from Frontend (Commands)

### Rust side (`lib.rs`):
```rust
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Frontend side (TypeScript):
```typescript
import { invoke } from '@tauri-apps/api/core';

const greeting = await invoke<string>('greet', { name: 'World' });
```

### With error handling:
```rust
#[tauri::command]
fn my_command() -> Result<String, String> {
    // Return Ok or Err
    Ok("success".into())
}
```

### Async commands:
```rust
#[tauri::command]
async fn async_command() -> Result<String, String> {
    // Can use .await here
    Ok("done".into())
}
```

---

## State Management

### Define and manage state:
```rust
use std::sync::Mutex;
use tauri::Manager;

struct AppState {
    is_recording: Mutex<bool>,
    clip_count: Mutex<u32>,
}

pub fn run() {
    tauri::Builder::default()
        .manage(AppState {
            is_recording: Mutex::new(false),
            clip_count: Mutex::new(0),
        })
        .invoke_handler(tauri::generate_handler![get_status])
        .run(tauri::generate_context!())
        .expect("error");
}

#[tauri::command]
fn get_status(state: tauri::State<'_, AppState>) -> bool {
    *state.is_recording.lock().unwrap()
}
```

---

## System Tray

### Enable in Cargo.toml:
```toml
tauri = { version = "2", features = ["tray-icon"] }
```

### Rust setup:
```rust
use tauri::{
    Manager,
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
};

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Create menu items
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let save_clip = MenuItem::with_id(app, "save_clip", "Save Clip", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&save_clip, &quit_i])?;

            // Build tray
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "quit" => {
                        app.exit(0);
                    }
                    "save_clip" => {
                        println!("Save clip triggered");
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
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

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error");
}
```

---

## Global Shortcuts

### Install:
```bash
npm run tauri add global-shortcut
# This auto-adds to Cargo.toml + package.json
```

### Rust setup:
```rust
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState
};

// In .setup():
let save_shortcut = Shortcut::new(Some(Modifiers::CONTROL), Code::F9);

app.handle().plugin(
    tauri_plugin_global_shortcut::Builder::new()
        .with_handler(move |_app, shortcut, event| {
            if shortcut == &save_shortcut {
                if let ShortcutState::Pressed = event.state() {
                    println!("Ctrl+F9 pressed - save clip!");
                }
            }
        })
        .build(),
)?;

app.global_shortcut().register(save_shortcut)?;
```

### Capabilities needed:
```json
{
  "permissions": [
    "global-shortcut:allow-register",
    "global-shortcut:allow-unregister",
    "global-shortcut:allow-is-registered"
  ]
}
```
