# SD Cards Manager

A fast, lightweight desktop application for managing files on SD cards and external storage devices. Built with Tauri, SvelteKit, and Rust for optimal performance and a native feel.

## Features

- 📁 **Quick Mount Detection** - Automatically detects all available drives and mount points
- 🔍 **Fast File Browsing** - Recursively lists all files in a directory with lightning-fast performance
- 🎯 **Filter & Search** - Real-time file filtering to quickly find what you need
- 📝 **File Operations**:
  - Rename files
  - Delete files
  - Move files to different folders
  - Create new folders
- 👁️ **Hidden Files Toggle** - Show or hide hidden files with one click
- 🌓 **Dark Mode** - Eye-friendly dark theme with automatic system preference detection
- ⌨️ **Keyboard Shortcuts** - Efficient workflow with keyboard navigation
- 💾 **Safe Operations** - Path validation to prevent accidental file system escapes

## How to Use

### Getting Started

1. **Launch the application**
2. **Select a storage device** from the dropdown menu or click "Open" to browse for a folder
3. The app will scan and display all files in the selected location

### File Operations

- **Filter Files**: Type in the search box to filter files by name or path
- **Rename**: Click the pencil icon next to any file to rename it
- **Move**: Enter a destination folder path (e.g., `photos/2024`) in the "Move target" field, then click the folder icon on any file
- **Delete**: Click the trash icon to delete a file (with confirmation)
- **Create Folder**: Enter a folder path in "New folder name" and click "Create Folder"

### Keyboard Shortcuts

- `Ctrl+R` - Refresh mount points
- `Ctrl+O` - Open directory dialog
- `Ctrl+F` - Focus search/filter box
- `Ctrl+H` - Toggle hidden files visibility
- `Ctrl+D` - Toggle dark/light theme

### File Information

Each file displays:
- **Filename** - The name of the file
- **Path** - The relative directory path from the root
- **Size** - File size in human-readable format (B, KB, MB, GB)

## Building the App

### Prerequisites

- [Node.js](https://nodejs.org/) (v16 or later)
- [Rust](https://www.rust-lang.org/tools/install)
- Platform-specific dependencies for Tauri

### Development

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev
```

### Production Build

```bash
# Build the application
npm run tauri build
```

The installer will be generated in:
- **Windows**: `src-tauri\target\release\bundle\nsis\` (`.exe` installer)
- **macOS**: `src-tauri/target/release/bundle/dmg/` (`.dmg` file)
- **Linux**: `src-tauri/target/release/bundle/appimage/` (`.AppImage` file)

## Technology Stack

- **Frontend**: SvelteKit + Vite
- **Backend**: Rust + Tauri
- **UI**: Custom CSS with light/dark themes

## Recommended IDE Setup

[VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
