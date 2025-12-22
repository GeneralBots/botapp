# BotApp Development Prompt Guide

**Version:** 6.1.0  
**Purpose:** LLM context for BotApp desktop application development

---

## ZERO TOLERANCE POLICY

**This project has the strictest code quality requirements possible.**

**EVERY SINGLE WARNING MUST BE FIXED. NO EXCEPTIONS.**

---

## ABSOLUTE PROHIBITIONS

```
❌ NEVER use #![allow()] or #[allow()] in source code to silence warnings
❌ NEVER use _ prefix for unused variables - DELETE the variable or USE it
❌ NEVER use .unwrap() - use ? or proper error handling
❌ NEVER use .expect() - use ? or proper error handling  
❌ NEVER use panic!() or unreachable!() - handle all cases
❌ NEVER use todo!() or unimplemented!() - write real code
❌ NEVER leave unused imports - DELETE them
❌ NEVER leave dead code - DELETE it or IMPLEMENT it
❌ NEVER use approximate constants (3.14159) - use std::f64::consts::PI
❌ NEVER silence clippy in code - FIX THE CODE or configure in Cargo.toml
❌ NEVER add comments explaining what code does - code must be self-documenting
```

---

## CARGO.TOML LINT EXCEPTIONS

When a clippy lint has **technical false positives** that cannot be fixed in code,
disable it in `Cargo.toml` with a comment explaining why:

```toml
[lints.clippy]
# Disabled: has false positives for functions with mut self, heap types (Vec, String)
missing_const_for_fn = "allow"
# Disabled: Tauri commands require owned types (Window) that cannot be passed by reference
needless_pass_by_value = "allow"
# Disabled: transitive dependencies we cannot control
multiple_crate_versions = "allow"
```

**Approved exceptions:**
- `missing_const_for_fn` - false positives for `mut self`, heap types
- `needless_pass_by_value` - Tauri/framework requirements
- `multiple_crate_versions` - transitive dependencies
- `future_not_send` - when async traits require non-Send futures

---

## MANDATORY CODE PATTERNS

### Error Handling - Use `?` Operator

```rust
// ❌ WRONG
let value = something.unwrap();
let value = something.expect("msg");

// ✅ CORRECT
let value = something?;
let value = something.ok_or_else(|| Error::NotFound)?;
```

### Self Usage in Impl Blocks

```rust
// ❌ WRONG
impl MyStruct {
    fn new() -> MyStruct { MyStruct { } }
}

// ✅ CORRECT
impl MyStruct {
    fn new() -> Self { Self { } }
}
```

### Format Strings - Inline Variables

```rust
// ❌ WRONG
format!("Hello {}", name)

// ✅ CORRECT
format!("Hello {name}")
```

### Derive Eq with PartialEq

```rust
// ❌ WRONG
#[derive(PartialEq)]
struct MyStruct { }

// ✅ CORRECT
#[derive(PartialEq, Eq)]
struct MyStruct { }
```

---

## Weekly Maintenance - EVERY MONDAY

### Package Review Checklist

**Every Monday, review the following:**

1. **Dependency Updates**
   ```bash
   cargo outdated
   cargo audit
   ```

2. **Package Consolidation Opportunities**
   - Check if new crates can replace custom code
   - Look for crates that combine multiple dependencies
   - Review `Cargo.toml` for redundant dependencies

3. **Code Reduction Candidates**
   - Custom implementations that now have crate equivalents
   - Boilerplate that can be replaced with derive macros
   - Tauri plugin replacements for custom code

4. **Tauri Plugin Updates**
   ```bash
   # Check for new Tauri plugins that simplify code
   # Review tauri-plugin-* ecosystem
   ```

### Packages to Watch

| Area | Potential Packages | Purpose |
|------|-------------------|---------|
| Dialogs | `tauri-plugin-dialog` | Native file dialogs |
| Notifications | `tauri-plugin-notification` | System notifications |
| Clipboard | `tauri-plugin-clipboard` | Clipboard access |
| Auto-update | `tauri-plugin-updater` | App updates |

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
botserver API
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
BOTSERVER_URL=http://localhost:8081  # botserver location
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

## Remember

- **ZERO WARNINGS** - Every clippy warning must be fixed
- **NO ALLOW IN CODE** - Never use #[allow()] in source files
- **CARGO.TOML EXCEPTIONS OK** - Disable lints with false positives in Cargo.toml with comment
- **NO DEAD CODE** - Delete unused code, never prefix with _
- **NO UNWRAP/EXPECT** - Use ? operator or proper error handling
- **INLINE FORMAT ARGS** - format!("{name}") not format!("{}", name)
- **USE SELF** - In impl blocks, use Self not the type name
- **DERIVE EQ** - Always derive Eq with PartialEq
- **USE DIAGNOSTICS** - Use IDE diagnostics tool, never call cargo clippy directly
- **Desktop-only features** - Shared logic in botserver
- **Tauri APIs** - No direct fs access from JS
- **Platform abstractions** - Use cfg for platform code
- **Security** - Minimal allowlist in tauri.conf.json
- **Version**: Always 6.1.0 - do not change without approval
- **Session Continuation**: When running out of context, create detailed summary: (1) what was done, (2) what remains, (3) specific files and line numbers, (4) exact next steps.
