# BotApp - General Bots Desktop Application

BotApp is the Tauri-based desktop wrapper for General Bots, providing native desktop and mobile capabilities on top of the pure web UI from [botui](https://github.com/GeneralBots/botui).

## Architecture

```
botui (pure web)          botapp (Tauri wrapper)
┌─────────────────┐      ┌─────────────────────────┐
│  suite/         │◄─────│  Loads botui's UI       │
│  minimal/       │      │  + injects app-only     │
│  shared/        │      │    features via JS      │
│                 │      │                         │
│  No Tauri deps  │      │  Tauri + native APIs    │
└─────────────────┘      └─────────────────────────┘
```

### Why Two Projects?

- **botui**: Pure web UI with zero native dependencies. Works in any browser.
- **botapp**: Wraps botui with Tauri for desktop/mobile native features.

This separation allows:
- Same UI code for web, desktop, and mobile
- Clean dependency management (web users don't need Tauri)
- App-specific features only in the native app

## Features

BotApp adds these native capabilities to botui:

- **Local File Access**: Browse and manage files on your device
- **System Tray**: Minimize to tray, background operation
- **Native Dialogs**: File open/save dialogs
- **Desktop Notifications**: Native OS notifications
- **App Settings**: Desktop-specific configuration

## Project Structure

```
botapp/
├── Cargo.toml              # Rust dependencies (includes Tauri)
├── build.rs                # Tauri build script
├── tauri.conf.json         # Tauri configuration
├── src/
│   ├── main.rs             # Tauri entry point
│   ├── lib.rs              # Library exports
│   └── desktop/
│       ├── mod.rs          # Desktop module
│       ├── drive.rs        # File system commands
│       └── tray.rs         # System tray functionality
├── ui/
│   └── app-guides/         # App-only HTML content
│       ├── local-files.html
│       └── native-settings.html
└── js/
    └── app-extensions.js   # Injected into botui's suite
```

## Prerequisites

- Rust 1.70+
- Node.js 18+ (for Tauri CLI)
- Tauri CLI: `cargo install tauri-cli`

### Platform-specific

**Linux:**
```bash
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev
```

**macOS:**
```bash
xcode-select --install
```

**Windows:**
- Visual Studio Build Tools with C++ workload

## Development

1. Clone both repositories:
```bash
git clone https://github.com/GeneralBots/botui.git
git clone https://github.com/GeneralBots/botapp.git
```

2. Start botui's web server (required for dev):
```bash
cd botui
cargo run
```

3. Run botapp in development mode:
```bash
cd botapp
cargo tauri dev
```

## Building

### Debug Build
```bash
cargo tauri build --debug
```

### Release Build
```bash
cargo tauri build
```

Binaries will be in `target/release/bundle/`.

## How App Extensions Work

BotApp injects `js/app-extensions.js` into botui's suite at runtime. This script:

1. Detects Tauri environment (`window.__TAURI__`)
2. Injects app-only navigation items into the suite's `.app-grid`
3. Exposes `window.BotApp` API for native features

Example usage in suite:
```javascript
if (window.BotApp?.isApp) {
    // Running in desktop app
    const files = await BotApp.fs.listFiles('/home/user');
    await BotApp.notify('Title', 'Native notification!');
}
```

## Tauri Commands

Available Tauri commands (invokable from JS):

| Command | Description |
|---------|-------------|
| `list_files` | List directory contents |
| `upload_file` | Copy file with progress |
| `create_folder` | Create new directory |
| `delete_path` | Delete file or folder |
| `get_home_dir` | Get user's home directory |

## Configuration

Edit `tauri.conf.json` to customize:

- `productName`: Application name
- `identifier`: Unique app identifier
- `build.devUrl`: URL for development (default: `http://localhost:3000`)
- `build.frontendDist`: Path to botui's UI (default: `../botui/ui/suite`)

## License

AGPL-3.0 - See [LICENSE](LICENSE) for details.

## Testing and Safety Tooling

BotApp follows General Bots' commitment to code quality and safety. The following tools are available for verification:

### Standard Testing

```bash
cargo test
```

### Miri (Undefined Behavior Detection)

Miri detects undefined behavior in unsafe code. Useful for testing data structures and parsing logic.

```bash
cargo +nightly miri test
```

**Limitations:** Cannot test I/O, FFI, or full integration tests.

### AddressSanitizer

Detects memory errors at runtime:

```bash
RUSTFLAGS="-Z sanitizer=address" cargo +nightly test
```

### Kani (Formal Verification)

For mathematically proving critical code properties:

```bash
cargo kani --function critical_function
```

### Ferrocene

Ferrocene is a qualified Rust compiler for safety-critical systems (ISO 26262, IEC 61508).

**Should BotApp use Ferrocene?**

- **For typical desktop deployment:** No - standard Rust + testing is sufficient
- **Consider Ferrocene if:** Deploying in regulated industries (medical, automotive, aerospace)

For most use cases, comprehensive testing with the tools above provides adequate confidence.

See [Testing & Safety Tooling](../botbook/src/07-gbapp/testing-safety.md) for complete documentation.

## Related Projects

- [botui](https://github.com/GeneralBots/botui) - Pure web UI
- [botserver](https://github.com/GeneralBots/botserver) - Backend server
- [botlib](https://github.com/GeneralBots/botlib) - Shared Rust library