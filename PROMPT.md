# BotApp Development Prompt Guide

**Version:** 6.1.0  
**Purpose:** LLM context for BotApp desktop application development

---

## Official Icons - MANDATORY

**NEVER generate icons with LLM. ALWAYS use official SVG icons from assets.**

Icons are stored in:
- `botui/ui/suite/assets/icons/` - Runtime icons (source of truth)

### Available App Icons

| Icon | File | Usage |
|------|------|-------|
| Logo | `gb-logo.svg` | Main GB branding |
| Bot | `gb-bot.svg` | Bot/assistant |
| Analytics | `gb-analytics.svg` | Charts, dashboards |
| Calendar | `gb-calendar.svg` | Scheduling |
| Chat | `gb-chat.svg` | Messaging |
| Compliance | `gb-compliance.svg` | Security |
| Designer | `gb-designer.svg` | Workflows |
| Drive | `gb-drive.svg` | File storage |
| Mail | `gb-mail.svg` | Email |
| Meet | `gb-meet.svg` | Video calls |
| Paper | `gb-paper.svg` | Documents |
| Research | `gb-research.svg` | Search |
| Sources | `gb-sources.svg` | Knowledge |
| Tasks | `gb-tasks.svg` | Task management |

### Icon Guidelines

- All icons use `stroke="currentColor"` for theming
- ViewBox: `0 0 24 24`
- Stroke width: `1.5`
- Rounded line caps and joins

**DO NOT:**
- Generate new icons with AI/LLM
- Use emoji or unicode symbols as icons
- Use external icon libraries
- Create inline SVG content

---

## Project Overview

BotApp is a **Tauri-based desktop wrapper** for General Bots. It provides native desktop experience by wrapping botui's web interface with Tauri's native window capabilities.

### Workspace Position

```
botapp/        # THIS PROJECT - Desktop app wrapper
botui/         # Web UI (consumed by botapp)
botserver/     # Main server (business logic)
botlib/        # Shared library
botbook/       # Documentation
```

### What BotApp Provides

- **Native Desktop Window**: Tauri-powered native application
- **System Tray**: Background operation with tray icon
- **File Dialogs**: Native file picker integration
- **Desktop Notifications**: OS-level notifications
- **Auto-Update**: Built-in update mechanism (future)

---

## Quick Start

```bash
# Ensure botserver is running
cd ../botserver && cargo run &

# Development mode
cd botapp
cargo tauri dev

# Production build
cargo tauri build
```

---

## Architecture

### Tauri Structure

```
botapp/
├── src/
│   └── main.rs           # Rust backend, Tauri commands
├── ui/                   # Frontend assets
│   └── app-guides/       # App-specific HTML
├── js/
│   └── app-extensions.js # JavaScript extensions
├── icons/                # App icons (all sizes)
├── tauri.conf.json       # Tauri configuration
├── Cargo.toml            # Rust dependencies
└── build.rs              # Build script
```

### Communication Flow

```
Native UI (HTML/CSS/JS)
    ↓ Tauri IPC (invoke)
Rust #[tauri::command]
    ↓ HTTP (reqwest)
BotServer API
    ↓
Business Logic + Database
```

---

## Code Generation Rules

### CRITICAL REQUIREMENTS

```
- Tauri commands must be async-safe
- All file operations use Tauri APIs
- No direct filesystem access from JS
- Desktop-specific features only in botapp
- Shared logic stays in botserver
- Zero warnings required
```

### Tauri Command Pattern

```rust
use tauri::command;

#[command]
pub async fn my_command(
    window: tauri::Window,
    param: String,
) -> Result<MyResponse, String> {
    // Implementation
    Ok(MyResponse { /* ... */ })
}

// Register in main.rs
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            my_command,
        ])
        .run(tauri::generate_context!())
        .expect("error running app");
}
```

### JavaScript Invocation

```javascript
// From frontend
const result = await window.__TAURI__.invoke('my_command', {
    param: 'value'
});
```

---

## Feature Flags

```toml
[features]
default = ["desktop"]
desktop = []
desktop-tray = ["dep:ksni", "dep:trayicon"]
```

---

## Dependencies

| Library | Version | Purpose |
|---------|---------|---------|
| tauri | 2 | Desktop framework |
| tauri-plugin-dialog | 2 | File dialogs |
| tauri-plugin-opener | 2 | URL/file opener |
| botlib | path | Shared types |
| reqwest | 0.12 | HTTP client |
| tokio | 1.41 | Async runtime |

---

## Platform-Specific Code

### Unix (Linux/macOS)

```rust
#[cfg(unix)]
use ksni;  // System tray on Linux
```

### Windows

```rust
#[cfg(windows)]
use trayicon;  // System tray on Windows
use image;     // Icon handling
```

---

## Tauri Configuration (tauri.conf.json)

Key settings (Tauri v2 format):

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "General Bots",
  "version": "6.1.0",
  "identifier": "br.com.pragmatismo.botapp",
  "build": {
    "devUrl": "http://localhost:3000",
    "frontendDist": "../botui/ui/suite"
  },
  "app": {
    "security": {
      "csp": null
    },
    "windows": [{
      "title": "General Bots",
      "width": 1200,
      "height": 800,
      "resizable": true,
      "fullscreen": false
    }]
  },
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": []
  }
}
```

---

## Adding Features

### Process

1. **Check if feature belongs here** - Only desktop-specific features
2. **Add Tauri command** in `src/main.rs`
3. **Register handler** in `tauri::Builder`
4. **Add JS invocation** in `js/app-extensions.js`
5. **Update UI** if needed

### Example: Add Screenshot

```rust
// src/main.rs
#[tauri::command]
async fn take_screenshot(window: tauri::Window) -> Result<Vec<u8>, String> {
    // Use platform-specific screenshot API
    Ok(screenshot_bytes)
}
```

```javascript
// js/app-extensions.js
async function takeScreenshot() {
    return await window.__TAURI__.invoke('take_screenshot');
}
```

---

## Icons

Required icon sizes in `icons/`:

```
icon.ico          # Windows (256x256)
icon.icns         # macOS
icon.png          # Linux (512x512)
32x32.png
128x128.png
128x128@2x.png
```

---

## Building

### Development

```bash
cargo tauri dev
```

### Production

```bash
# All platforms
cargo tauri build

# Specific target
cargo tauri build --target x86_64-unknown-linux-gnu
cargo tauri build --target x86_64-pc-windows-msvc
cargo tauri build --target x86_64-apple-darwin
```

### Output Locations

```
target/release/bundle/
├── deb/          # Debian package
├── appimage/     # AppImage
├── msi/          # Windows installer
├── dmg/          # macOS disk image
└── macos/        # macOS app bundle
```

---

## Environment Variables

```bash
BOTSERVER_URL=http://localhost:8081  # BotServer location
TAURI_DEBUG=1                         # Debug mode
```

---

## Testing

```bash
# Build check
cargo build

# Run dev mode
cargo tauri dev

# Run tests
cargo test
```

---

## Rules

- **Desktop-only features** - Shared logic in botserver
- **Tauri APIs** - No direct fs access from JS
- **Platform abstractions** - Use cfg for platform code
- **Security** - Minimal allowlist in tauri.conf.json
- **Zero warnings** - Clean compilation required
- **No cargo audit** - Exempt per project requirements
