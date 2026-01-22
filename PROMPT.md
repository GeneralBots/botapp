# BotApp Development Guide

**Version:** 6.2.0  
**Purpose:** Desktop application wrapper (Tauri 2)

---

## ZERO TOLERANCE POLICY

**EVERY SINGLE WARNING MUST BE FIXED. NO EXCEPTIONS.**

---

## ❌ ABSOLUTE PROHIBITIONS

```
❌ NEVER use #![allow()] or #[allow()] in source code
❌ NEVER use _ prefix for unused variables - DELETE or USE them
❌ NEVER use .unwrap() - use ? or proper error handling
❌ NEVER use .expect() - use ? or proper error handling  
❌ NEVER use panic!() or unreachable!()
❌ NEVER use todo!() or unimplemented!()
❌ NEVER leave unused imports or dead code
❌ NEVER add comments - code must be self-documenting
```

---

## 🔐 SECURITY - TAURI SPECIFIC

```
❌ NEVER trust user input from IPC commands
❌ NEVER expose filesystem paths to frontend without validation
❌ NEVER store secrets in plain text or localStorage
❌ NEVER disable CSP in tauri.conf.json for production
❌ NEVER use allowlist: all in Tauri configuration
```

### Path Validation

```rust
// ❌ WRONG - trusting user path
#[tauri::command]
async fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| e.to_string())
}

// ✅ CORRECT - validate and sandbox paths
#[tauri::command]
async fn read_file(app: tauri::AppHandle, filename: String) -> Result<String, String> {
    let safe_name = filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-')
        .collect::<String>();
    if safe_name.contains("..") {
        return Err("Invalid filename".into());
    }
    let base_dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let full_path = base_dir.join(&safe_name);
    std::fs::read_to_string(full_path).map_err(|e| e.to_string())
}
```

---

## 🏗️ ARCHITECTURE

### Structure

```
botapp/
├── src/
│   └── main.rs           # Rust backend, Tauri commands
├── ui/                   
│   └── app-guides/       # App-specific HTML
├── js/
│   └── app-extensions.js # JavaScript extensions
├── icons/                # App icons (all sizes)
├── tauri.conf.json       # Tauri configuration
└── Cargo.toml
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

## 🔧 TAURI COMMAND PATTERN

```rust
use tauri::command;

#[command]
pub async fn my_command(
    window: tauri::Window,
    param: String,
) -> Result<MyResponse, String> {
    if param.is_empty() || param.len() > 1000 {
        return Err("Invalid parameter".into());
    }
    Ok(MyResponse { /* ... */ })
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            my_command,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| format!("error running app: {e}"))?;
}
```

### JavaScript Invocation

```javascript
const result = await window.__TAURI__.invoke('my_command', {
    param: 'value'
});
```

---

## 🎨 ICONS - MANDATORY

**NEVER generate icons with LLM. Use official SVG icons from `botui/ui/suite/assets/icons/`**

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

## ⚙️ CONFIGURATION (tauri.conf.json)

```json
{
  "$schema": "https://schema.tauri.app/config/2",
  "productName": "General Bots",
  "version": "6.2.0",
  "identifier": "br.com.pragmatismo.botapp",
  "build": {
    "devUrl": "http://localhost:3000",
    "frontendDist": "../botui/ui/suite"
  },
  "app": {
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'"
    }
  }
}
```

---

## 📦 KEY DEPENDENCIES

| Library | Version | Purpose |
|---------|---------|---------|
| tauri | 2 | Desktop framework |
| tauri-plugin-dialog | 2 | File dialogs |
| tauri-plugin-opener | 2 | URL/file opener |
| botlib | path | Shared types |
| reqwest | 0.12 | HTTP client |
| tokio | 1.41 | Async runtime |

---

## 🔑 REMEMBER

- **ZERO WARNINGS** - Every clippy warning must be fixed
- **NO ALLOW IN CODE** - Never use #[allow()] in source files
- **NO DEAD CODE** - Delete unused code
- **NO UNWRAP/EXPECT** - Use ? operator
- **Security** - Minimal allowlist, validate ALL inputs
- **Desktop-only features** - Shared logic in botserver
- **Tauri APIs** - No direct fs access from JS
- **Version 6.2.0** - do not change without approval