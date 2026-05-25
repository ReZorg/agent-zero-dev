# Agent Zero Desktop Application

This document provides comprehensive documentation for building, distributing, and using the Agent Zero desktop application.

## Table of Contents

1. [Overview](#overview)
2. [Installation](#installation)
3. [Building from Source](#building-from-source)
4. [Configuration](#configuration)
5. [Architecture](#architecture)
6. [Troubleshooting](#troubleshooting)
7. [Development Guide](#development-guide)

## Overview

The Agent Zero Desktop application provides a native desktop experience for running Agent Zero without requiring Docker or manual Python setup. It bundles:

- **Tauri Shell**: A lightweight Rust-based application shell (~10MB)
- **Python Backend**: The complete Agent Zero server bundled via PyInstaller
- **WebUI**: The frontend assets served locally

### Supported Platforms

| Platform | Architecture | Installer Format |
|----------|-------------|------------------|
| Windows | x64 | `.msi`, `.exe` |
| macOS | Intel (x64) | `.dmg`, `.app` |
| macOS | Apple Silicon (arm64) | `.dmg`, `.app` |
| Linux | x64 | `.deb`, `.AppImage`, `.rpm` |

### System Requirements

- **RAM**: 4GB minimum, 8GB+ recommended
- **Storage**: ~1GB for application + model cache
- **OS Versions**:
  - Windows 10/11
  - macOS 10.15 (Catalina) or later
  - Ubuntu 20.04+ / Debian 11+ / Fedora 36+

## Installation

### Windows

1. Download `Agent-Zero_x.x.x_x64.msi` from the [Releases](https://github.com/agent0ai/agent-zero/releases) page
2. Run the installer and follow the prompts
3. Launch "Agent Zero" from the Start Menu

### macOS

1. Download `Agent-Zero_x.x.x_x64.dmg` (Intel) or `Agent-Zero_x.x.x_aarch64.dmg` (Apple Silicon)
2. Open the DMG and drag Agent Zero to Applications
3. On first launch, right-click and select "Open" to bypass Gatekeeper (if unsigned)

### Linux

**Debian/Ubuntu (.deb)**:
```bash
sudo dpkg -i agent-zero_x.x.x_amd64.deb
sudo apt-get install -f  # Install dependencies
```

**Fedora/RHEL (.rpm)**:
```bash
sudo rpm -i agent-zero-x.x.x-1.x86_64.rpm
```

**AppImage** (universal):
```bash
chmod +x Agent-Zero_x.x.x_amd64.AppImage
./Agent-Zero_x.x.x_amd64.AppImage
```

## Building from Source

### Prerequisites

1. **Node.js 18+** and npm
2. **Rust 1.70+** via [rustup](https://rustup.rs)
3. **Python 3.12+**
4. Platform-specific build tools (see [desktop/README.md](../desktop/README.md))

### Build Steps

```bash
# Clone the repository
git clone https://github.com/agent0ai/agent-zero.git
cd agent-zero

# Install Python dependencies
pip install -r requirements.txt
pip install -r requirements2.txt
pip install pyinstaller>=6.0.0

# Install Tauri dependencies
cd desktop
npm install

# Build the Python backend (sidecar)
npm run build:python

# Build the desktop application
npm run tauri:build
```

The built installers will be in `desktop/src-tauri/target/release/bundle/`.

## Configuration

### First Launch Setup

On first launch, Agent Zero will:
1. Create a configuration directory:
   - Windows: `%APPDATA%\ai.agent0.desktop\`
   - macOS: `~/Library/Application Support/ai.agent0.desktop/`
   - Linux: `~/.config/ai.agent0.desktop/`
2. Initialize default settings
3. Open the main window once the backend is ready

### Environment Variables

Configure Agent Zero using environment variables or a `.env` file in the config directory:

```env
# LLM Configuration
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
OPENROUTER_API_KEY=...

# Server Settings
WEB_UI_PORT=50001

# Authentication (optional)
AUTH_LOGIN=admin
AUTH_PASSWORD=secret
```

### Settings via UI

Most settings can be configured through the Settings modal in the application (⚙️ icon).

## Architecture

### Process Model

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri Process                        │
│  (Rust binary, manages lifecycle and native features)   │
└────────────────────────┬────────────────────────────────┘
                         │ Spawns on startup
                         ▼
┌─────────────────────────────────────────────────────────┐
│                   Python Sidecar                        │
│  (PyInstaller bundle running Flask/Uvicorn server)      │
│                                                         │
│  • Listens on localhost:50001                           │
│  • Outputs "DESKTOP_READY" signal when ready            │
│  • Terminated when Tauri window closes                  │
└─────────────────────────────────────────────────────────┘
```

### Communication Flow

1. **Startup**: Tauri spawns the Python sidecar with `--desktop-mode`
2. **Health Check**: Tauri polls `/api/health` until backend responds
3. **Ready Signal**: Backend prints `DESKTOP_READY`, Tauri shows the window
4. **Runtime**: WebView loads `http://localhost:50001`, communicates via HTTP/WebSocket
5. **Shutdown**: Closing the window terminates both processes

### IPC Commands

The Tauri shell exposes these commands to the frontend:

| Command | Description |
|---------|-------------|
| `get_backend_status` | Returns backend running state, PID, port |
| `get_app_info` | Returns app version, platform, architecture |
| `open_external_url` | Opens URL in system browser |
| `check_backend_health` | Pings the health endpoint |
| `show_main_window` | Shows and focuses the main window |

## Troubleshooting

### Backend Won't Start

**Symptoms**: Window stays blank or shows loading indefinitely

**Solutions**:
1. Check if port 50001 is in use:
   ```bash
   # Windows
   netstat -ano | findstr :50001
   
   # macOS/Linux
   lsof -i :50001
   ```
2. Kill any conflicting processes
3. Check application logs:
   - Windows: `%LOCALAPPDATA%\ai.agent0.desktop\logs\`
   - macOS: `~/Library/Logs/ai.agent0.desktop/`
   - Linux: `~/.local/share/ai.agent0.desktop/logs/`

### High Memory Usage

The Python backend may use significant memory due to:
- Embedding models (~500MB)
- LLM context caching
- Document processing

To reduce memory:
1. Use smaller embedding models in settings
2. Clear chat history periodically
3. Restart the application if memory exceeds acceptable levels

### Security Warnings (Unsigned Builds)

Development builds are not code-signed. To run:

**Windows**: Click "More info" → "Run anyway" on SmartScreen prompt

**macOS**: Right-click the app → "Open" → "Open" in the dialog

### Linux: Missing Libraries

If you see WebKit or GTK errors:

```bash
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.1-0 libayatana-appindicator3-1 libssl3

# Fedora
sudo dnf install webkit2gtk4.1 libappindicator-gtk3 openssl
```

## Development Guide

### Local Development

```bash
cd desktop
npm run tauri:dev
```

This runs the Tauri development server, which:
- Watches for Rust source changes and recompiles
- Serves the WebUI from the parent `webui/` directory
- Hot-reloads frontend changes

### Testing the Sidecar

Test the Python sidecar independently:

```bash
cd ..
python run_ui.py --desktop-mode --port 50001 --host 127.0.0.1
```

### Adding IPC Commands

1. Add the command function in `src-tauri/src/commands.rs`
2. Register it in `main.rs` with `tauri::generate_handler!`
3. Call from frontend using `@tauri-apps/api`:
   ```javascript
   import { invoke } from '@tauri-apps/api/core';
   const status = await invoke('get_backend_status');
   ```

### Building Debug Versions

```bash
npm run tauri:build:debug
```

Debug builds include:
- Developer tools (F12)
- Verbose logging
- Unminified code

### Release Process

1. Update version in `desktop/src-tauri/tauri.conf.json`
2. Update version in `desktop/package.json`
3. Commit changes
4. Create and push a tag: `git tag desktop-v1.2.3 && git push --tags`
5. GitHub Actions will build and publish the release

## Auto-Updates

The application checks for updates on startup (when configured). Updates are:
1. Downloaded in the background
2. Verified via signature
3. Applied on next restart

To disable auto-updates, set in `tauri.conf.json`:
```json
"plugins": {
  "updater": {
    "active": false
  }
}
```

## Security Considerations

- The backend binds to `127.0.0.1` only (not network-accessible)
- API keys are stored in the local config directory
- Inter-process communication uses localhost HTTP
- For production distribution, implement code signing

## Contributing

See the main [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines on contributing to Agent Zero, including the desktop application.
