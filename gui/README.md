# PhiSHRI Installer GUI

A modern, clean GUI wrapper for the PhiSHRI MCP installer built with Tauri.

## Features

- Dark aesthetic theme matching PhiSHRI branding
- Multiple installation methods: Auto, mcpb, DXT, Manual
- Real-time progress tracking
- Installation status checking
- One-click install/uninstall

## Prerequisites

- [Node.js](https://nodejs.org/) v18+
- [Rust](https://rustup.rs/) (latest stable)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

## Quick Start

```powershell
# Install dependencies
cd gui
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

## Project Structure

```
gui/
├── src/                    # Web frontend
│   ├── index.html         # Main HTML
│   ├── styles.css         # Dark theme styles
│   └── main.js            # Frontend logic
├── src-tauri/             # Rust backend
│   ├── src/
│   │   └── main.rs        # Tauri commands
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri config
├── package.json           # Node dependencies
└── vite.config.js         # Vite bundler config
```

## Building

### Development

```powershell
npm run tauri dev
```

### Production Build

```powershell
npm run tauri build
```

The built installer will be in `src-tauri/target/release/bundle/`.

## Web Admin Extension

This GUI is designed to be extended into a web admin panel. The same web frontend can be served via a local HTTP server for browser-based management:

```powershell
# Future: Run as web admin
phishri-admin --web --port 8080
```

## Architecture

```
┌─────────────────────────────────────────┐
│           Tauri Window                   │
├─────────────────────────────────────────┤
│  Web Frontend (HTML/CSS/JS)              │
│  - Method selection UI                   │
│  - Progress display                      │
│  - Status indicators                     │
├─────────────────────────────────────────┤
│  Rust Backend                            │
│  - PowerShell integration                │
│  - File system operations                │
│  - Event emission                        │
├─────────────────────────────────────────┤
│  PowerShell Installer Script             │
│  - install.ps1 (existing)                │
│  - Multi-method support                  │
└─────────────────────────────────────────┘
```

## License

MIT
