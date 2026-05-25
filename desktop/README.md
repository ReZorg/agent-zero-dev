# Agent Zero Desktop

Native desktop application for Agent Zero, built with Tauri and PyInstaller.

## Overview

The desktop application bundles the Agent Zero backend as a sidecar process and provides a native desktop experience across Windows, macOS, and Linux.

## Architecture

```
┌─────────────────────────────────────────────────────┐
│                   Desktop App                       │
├─────────────────┬───────────────────────────────────┤
│   Tauri Shell   │       Python Sidecar              │
│   (Rust/WebView)│  (PyInstaller bundle)             │
│       │         │       │                           │
│   WebUI Assets  │   Flask + Uvicorn + Agent Zero    │
│   (Static HTML) │       │                           │
│       │         │   All Python dependencies         │
│       └─────────┼───────┘                           │
│           IPC   │  (localhost:50001)                │
│        (Tauri commands or HTTP/WebSocket)           │
└─────────────────────────────────────────────────────┘
```

## Prerequisites

### For Development

- **Node.js** 18+ and npm
- **Rust** 1.70+ (install via [rustup](https://rustup.rs))
- **Python** 3.12+
- Platform-specific dependencies:
  - **Linux**: `libwebkit2gtk-4.1-dev`, `libappindicator3-dev`, `librsvg2-dev`
  - **macOS**: Xcode Command Line Tools
  - **Windows**: Visual Studio Build Tools with C++ workload

### Installing Dependencies

```bash
# Install Node.js dependencies
cd desktop
npm install

# Install Python dependencies (for building the sidecar)
pip install -r ../requirements.txt
pip install -r ../requirements2.txt
pip install pyinstaller>=6.0.0
```

## Building

### Quick Build (Current Platform)

```bash
cd desktop

# Build Python backend
npm run build:python

# Build Tauri app
npm run tauri:build
```

### Platform-Specific Builds

```bash
# Windows
npm run build:python:windows
npm run tauri:build -- --target x86_64-pc-windows-msvc

# macOS (Intel)
npm run build:python:macos
npm run tauri:build -- --target x86_64-apple-darwin

# macOS (Apple Silicon)
npm run build:python:macos
npm run tauri:build -- --target aarch64-apple-darwin

# Linux
npm run build:python:linux
npm run tauri:build -- --target x86_64-unknown-linux-gnu
```

### Build Output

After building, installers are located in:
- `src-tauri/target/<target>/release/bundle/`

| Platform | Outputs |
|----------|---------|
| Windows | `.msi`, `.exe` |
| macOS | `.dmg`, `.app` |
| Linux | `.deb`, `.AppImage`, `.rpm` |

## Development

### Running in Development Mode

```bash
cd desktop
npm run tauri:dev
```

This starts the Tauri development server with hot-reload enabled for frontend changes.

### Project Structure

```
desktop/
├── src-tauri/           # Tauri (Rust) source
│   ├── Cargo.toml       # Rust dependencies
│   ├── tauri.conf.json  # Tauri configuration
│   ├── src/
│   │   ├── main.rs      # Entry point, backend management
│   │   ├── lib.rs       # Library exports
│   │   └── commands.rs  # IPC commands
│   ├── binaries/        # PyInstaller output (sidecar)
│   └── icons/           # App icons
├── src/                 # Frontend assets (copied from webui/)
├── scripts/
│   └── build_python.py  # PyInstaller build script
├── package.json         # npm scripts
└── README.md
```

## Configuration

### Tauri Configuration (`tauri.conf.json`)

Key settings:
- `productName`: Application name
- `identifier`: Unique app identifier (e.g., `ai.agent0.desktop`)
- `version`: App version
- `bundle.externalBin`: Sidecar binary path
- `plugins.updater`: Auto-update configuration

### Environment Variables

The desktop app inherits environment variables from the system. You can configure:
- `WEB_UI_PORT`: Backend port (default: 50001)
- `AUTH_LOGIN` / `AUTH_PASSWORD`: Authentication credentials
- LLM API keys (same as web version)

## Code Signing

### macOS

To sign and notarize for macOS distribution:

1. Obtain an Apple Developer certificate
2. Set environment variables:
   ```bash
   export APPLE_CERTIFICATE="base64-encoded-certificate"
   export APPLE_CERTIFICATE_PASSWORD="certificate-password"
   export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
   export APPLE_ID="your@apple.id"
   export APPLE_PASSWORD="app-specific-password"
   export APPLE_TEAM_ID="YOUR_TEAM_ID"
   ```

### Windows

For Windows code signing:

1. Obtain a code signing certificate (EV recommended for SmartScreen)
2. Set environment variables:
   ```bash
   export WINDOWS_CERTIFICATE="base64-encoded-certificate"
   export WINDOWS_CERTIFICATE_PASSWORD="certificate-password"
   ```

## Auto-Updates

The desktop app supports automatic updates via Tauri's updater plugin.

### Update Server

By default, updates are fetched from GitHub Releases. The update manifest (`latest.json`) is automatically generated during the release workflow.

### Manual Update Check

Users can check for updates via the app's settings menu (when implemented).

## CI/CD

The GitHub Actions workflow (`.github/workflows/desktop-build.yml`) handles:

1. **Build Python Backend**: Creates PyInstaller bundles for each platform
2. **Build Tauri App**: Compiles the Rust shell and bundles everything
3. **Release**: Creates GitHub Release with all installers

### Triggering Builds

- **Tag Push**: Push a tag like `desktop-v1.0.0` to trigger a release build
- **Manual Dispatch**: Use GitHub Actions UI to trigger builds with custom version

## Troubleshooting

### Common Issues

1. **"Backend failed to start"**
   - Check if port 50001 is available
   - Look for Python errors in the console/logs

2. **"Sidecar not found"**
   - Ensure Python backend was built for the correct platform
   - Check `src-tauri/binaries/` for the sidecar binary

3. **Linux: WebKit errors**
   - Install required dependencies: `libwebkit2gtk-4.1-dev`

4. **macOS: Notarization fails**
   - Ensure all signing credentials are correct
   - Check Apple Developer account status

### Debug Mode

Build with debug information:
```bash
npm run tauri:build:debug
```

## License

Same as the main Agent Zero project.
