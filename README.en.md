<div align="center">

# DiskAgent

**A lightweight disk usage analyzer for Windows, built with Rust and egui.**

**Language:** [Bahasa Indonesia](README.md) · English

[![Download DiskAgent.exe](https://img.shields.io/badge/Download-DiskAgent.exe-2ea44f?logo=windows&logoColor=white)](https://github.com/dimsyyy/DiskAgent/releases/latest/download/DiskAgent.exe)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-0078D6?logo=windows&logoColor=white)](#build-requirements)
[![Rust](https://img.shields.io/badge/Rust-stable-CE422B?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![egui](https://img.shields.io/badge/UI-egui%200.29-1f6feb)](https://github.com/emilk/egui)
[![Version](https://img.shields.io/badge/version-0.1.0-blue)](Cargo.toml)
[![License](https://img.shields.io/badge/license-MIT-2ea44f)](LICENSE)

[Download](#download) · [Features](#features) · [Screenshots](#screenshots) · [Installation](#installation) · [Build from source](#build-from-source) · [Usage](#usage) · [Keyboard](#keyboard-shortcuts) · [Architecture](#architecture) · [License](#license)

</div>

> **Status:** v0.1.0 — core features are working. Advanced features are not included yet.

## Download

[**Download DiskAgent.exe**](https://github.com/dimsyyy/DiskAgent/releases/latest/download/DiskAgent.exe) — Windows 10/11 64-bit.

No installer needed: download the file and run it. All versions are available on the [Releases](https://github.com/dimsyyy/DiskAgent/releases) page.

> [!NOTE]
> Release binaries are unsigned. Windows SmartScreen may show a warning on first launch; choose **More info → Run anyway** only if you trust the file from this official release page.

## About

DiskAgent is a Windows desktop app for finding which folders or files are using disk space. Pick a folder, run a scan, then explore the results in the **Details tree** or **Treemap Chart**.

DiskAgent has four main tabs:

- **Home** — start and stop scans, refresh, and open folders.
- **Scan** — recursive scanning with progress and status.
- **View** — Details, Treemap, units, columns, and sorting.
- **Options** — **English** and **Bahasa Indonesia** language options.

All interface icons are drawn in vector.

## Screenshots

![DiskAgent Treemap Chart](docs/screenshots/treemap.png)

<p align="center"><sub><strong>Treemap Chart</strong> — folder and file sizes shown as proportionally sized rectangles.</sub></p>

<details>
<summary><strong>View the Details screen</strong></summary>

![DiskAgent Details](docs/screenshots/details.png)

</details>

## Features

| Area | Capabilities |
| --- | --- |
| **Scanning** | Recursive folder scan on a background thread; responsive UI, live progress, and a cancellable scan. |
| **Details tree** | Name, Size, Allocated Space, Files, Folders, % of Parent, and Last Modified columns. Expand/collapse, sorting, and percentage bars. |
| **Treemap** | Squarified layout, color per item, click to select, double-click to zoom into a folder, plus Up/Root buttons. |
| **Filters & search** | Include/exclude masks, minimum size, file age, hide empty folders, and file/folder name search. |
| **Sorting & units** | Sort from column headers or the combo box; Auto/TB/GB/MB/KB/B units and decimal places. |
| **File actions** | Create a folder, open the location in Explorer, copy the path, export CSV, and permanent delete with confirmation. |
| **What is this folder?** | Right-click a folder/file to open Google Search with an automatic question about its purpose and safety. |
| **Appearance** | Ribbon, address bar, navigation buttons, status bar, configurable columns, and English/Indonesian UI. |
| **Windows integration** | Volume free space, cluster size, filesystem, elevated scan, and restart as Administrator. |

### Permanent delete

Right-click a file/folder in **Details** or **Treemap**, choose **Delete permanently**, then confirm. The item is removed immediately and **does not go to the Recycle Bin**. The folder currently being scanned cannot be deleted.

### What is this folder?

Right-click a folder/file, then choose **What is this folder?** or **What is this file?**. DiskAgent opens the default browser to Google Search with a question such as:

> What does the folder “node_modules” in “CODE\project” do, and is it safe to delete?

The query uses the item name and up to three trailing location components. Windows usernames are redacted, and file contents are never sent.

> [!NOTE]
> Google AI Overview is not guaranteed. Results may be ordinary articles, forum posts, or documentation. This feature needs an internet connection and opens a third-party service.

## Installation

There are two ways to run DiskAgent:

- **Download the binary** — fastest, no Rust required.
- **Build from source** — see [Build from source](#build-from-source).

### Download the binary

1. Open the [Releases](https://github.com/dimsyyy/DiskAgent/releases) page or click the **Download** button at the top.
2. Download `DiskAgent.exe`.
3. Run it directly; no installer is required.

### Build requirements

- Windows 10/11 64-bit.
- [Rust](https://rustup.rs/) stable with edition 2021.
- One of these linkers:
  - **MSVC** — Visual Studio Build Tools with the *Desktop development with C++* workload (recommended), or
  - **MinGW-w64** — `x86_64-w64-mingw32-gcc` available on `PATH`.

## Build from source

After cloning the repository:

```bat
cd DiskAgent
cargo build --release
```

The resulting binary:

```text
target\release\DiskAgent.exe
```

For a debug build:

```bat
cargo build
target\debug\DiskAgent.exe
```

### Using the MSVC toolchain

```bat
rustup default stable-x86_64-pc-windows-msvc
cargo build --release
```

### Using the GNU / MinGW-w64 toolchain

Make sure MinGW-w64 is installed and its `bin` folder is on `PATH`. Example:

```bat
rustup default stable-x86_64-pc-windows-gnu
set PATH=%USERPROFILE%\.cargo\bin;C:\mingw64\bin;%PATH%
cargo build --release
```

## Running

If you downloaded the binary from Releases, simply run `DiskAgent.exe` from your Downloads folder.

Start the app without arguments:

```bat
target\release\DiskAgent.exe
```

Or scan a specific folder immediately:

```bat
target\release\DiskAgent.exe "D:\Data"
```

## Usage

1. Click **Select Directory**, or type a path in the address bar.
2. Wait for the scan to finish. Progress and status appear at the bottom, and the scan can be stopped at any time.
3. Use the **Details** tab for the size table and **Treemap Chart** for the visual proportions.
4. Use **Filter** or `Ctrl+F` to narrow the results.
5. Right-click an item to open Explorer, copy its path, delete it permanently, or search for an explanation on Google.
6. Open **Options → Language** to switch the UI language.

### Settings

Settings are saved automatically at:

```text
%APPDATA%\DiskAgent\settings.json
```

Stored values: language, units, decimal places, visible columns, sorting, hide-empty, ribbon state, and last folder. Active filters are not persisted.

Delete that file to reset the app to its default settings.

## Keyboard shortcuts

| Key | Action |
| --- | --- |
| `↑` / `↓` | Move the selection |
| `→` | Expand a folder / move to its first child |
| `←` | Collapse a folder / move to its parent |
| `Enter` | Expand or collapse the selected folder |
| `Home` / `End` | Jump to the first / last row |
| `PageUp` / `PageDown` | Move the selection 20 rows |
| `Ctrl+F` | Open search |
| `Ctrl+C` | Copy the selected item's path |
| `Esc` | Close a menu/popup |

Double-clicking a folder expands/collapses it. Double-clicking a file selects that file in Explorer.

## Architecture

```text
src/
├── main.rs          # eframe entry point
├── app.rs           # App state, actions, keyboard handling, settings persistence
├── scan.rs          # Recursive scanner on a background thread
├── model.rs         # Node, ScanTree, aggregation, subtree removal
├── filter.rs        # Wildcard filters and ViewStats
├── format.rs        # Byte, count, percent, and unit formatting
├── platform.rs      # Windows FFI, volume info, elevation, delete, open URL
├── web.rs           # Google query builder and URL encoding
├── i18n.rs          # English/Indonesian translations
├── settings.rs      # JSON settings persistence
├── theme.rs         # Colors and style
└── ui/
    ├── ribbon.rs    # Tabs, button groups, address bar
    ├── details.rs   # Virtualized table and row interaction
    ├── treemap.rs   # Squarified treemap
    ├── statusbar.rs # Status bar
    ├── dialogs.rs   # Filter, columns, new folder, delete, search
    └── icons.rs     # Vector icons
```

### Technology

| Component | Technology |
| --- | --- |
| GUI | [eframe](https://github.com/emilk/eframe) 0.29 + [egui](https://github.com/emilk/egui) with the OpenGL (glow) backend |
| File dialogs | [rfd](https://github.com/PolyMeilex/rfd) |
| Settings | serde + serde_json |
| Scanner | `std::fs` + background thread + channel/atomic progress |
| Windows API | Manual FFI for volume info, elevation, ShellExecute, and timestamps |

## Roadmap

- [x] Recursive folder scanning with progress and cancellation.
- [x] Details tree, treemap, filters, search, sorting, and units.
- [x] English/Indonesian UI and persistent settings.
- [x] Permanent delete, folder creation, open in Explorer, and CSV export.
- [x] Google Search integration for **What is this folder?**.
- [x] v0.1.0 release binary on GitHub Releases.
- [ ] Advanced features (scheduled scans, duplicate finder, file age, top files).
- [ ] Windows installer and a custom app icon.
- [ ] CI and more unit tests.

## Limitations

- Scanning uses a recursive filesystem walk, not direct NTFS MFT parsing. Very large volumes may be slower than tools built specifically for NTFS.
- Allocated size is based on the volume cluster size; results can differ for compressed, encrypted, or reparse-point files.
- Advanced features such as scheduled scans, a duplicate finder, and file-age analysis are not available yet.
- **Permanent delete bypasses the Recycle Bin.** Back up important data before deleting.
- **What is this folder?** depends on Google and an internet connection; the result is not a guarantee that an item is safe to delete.
- Release binaries are unsigned, so Windows SmartScreen may show a warning on first launch.

## Contributing

Contributions are welcome for bug fixes, documentation, translations, and new features that fit the app's scope.

Before opening a pull request:

```bat
cargo fmt
cargo test
cargo build --release
```

For larger changes, please open an issue or discussion first so the implementation direction is clear.

## License

Released under the [MIT License](LICENSE).

## Acknowledgements

- Inspired by **TreeSize** from JAM Software.
- Built with Rust, egui/eframe, rfd, and serde.
- All UI icons are drawn in vector.

---

<p align="center"><sub>Built to answer the question: “What is this folder, and how much space is it using?”</sub></p>
