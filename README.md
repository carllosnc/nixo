<div align="center">

# Nixo Media Player

<p>
  <strong>High-performance cross-platform media player written in Rust</strong><br>
  100% native GPU-accelerated interface powered by <code>wgpu</code> + <code>egui</code> and hardware-accelerated playback via <code>libmpv</code>.
</p>

[![CI](https://github.com/carllosnc/nixo/actions/workflows/ci.yml/badge.svg)](https://github.com/carllosnc/nixo/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024%20edition-orange.svg)](Cargo.toml)

</div>

---

## Features

- **100% Native GUI (Zero Webview)**: Direct GPU rendering via Vulkan, DirectX 12, and Metal through `wgpu` and `egui`. Lightweight memory footprint and instantaneous startup.
- **Hardware-Accelerated Video Playback**: Backed by `libmpv` with automatic hardware decoding (`D3D11VA`, `NVDEC`, `VAAPI`, `VideoToolbox`) and pristine A/V synchronization.
- **Fallback Synthetic Engine**: Includes a built-in 60 FPS GPU test signal engine, allowing the full UI, texture pipeline, and seekbar to be tested even without dynamic libraries installed.
- **Decoupled Architecture**: Engine abstraction via the `MediaBackend` trait, enabling modular backends (MPV, GStreamer, Symphonia).
- **Native OS Dialogs & Drag-and-Drop**: Seamless file loading via native Explorer/Finder/GTK file dialogs (`rfd`) and drag-and-drop video file support.
- **Responsive On-Screen Display (OSD)**: Interactive playback bar, volume control, playback speed selector (0.5x - 2.0x), and auto-hiding controls in fullscreen mode.

---

## Project Architecture

The repository is organized as a modular **Cargo Workspace**:

```text
nixo/
├── Cargo.toml                 # Workspace root manifest & release profiles
├── assets/                    # Official application icons (ICO / PNG)
├── crates/
│   ├── nixo-core/             # Decoupled domain models, playback states, commands, and MediaBackend trait
│   ├── nixo-engine/           # Dynamic libmpv integration & synthetic 60 FPS GPU preview engine
│   └── nixo-app/              # Native GUI application (wgpu/egui window, OSD controls, shortcuts)
└── scripts/
    ├── setup-mpv.ps1          # Automatic libmpv setup script for Windows
    ├── package.ps1            # Standalone redistributable bundler (.zip + SHA256)
    └── generate-icon.ps1      # Procedural icon asset generator
```

---

## Getting Started

### 1. Prerequisites & libmpv Setup

Nixo dynamically links to `libmpv` at runtime, meaning **no C development headers or `.lib` files are required at compile time**.

- **Windows (PowerShell)**:
  Run the automated script to download and extract the official `libmpv-2.dll` into `vendor/mpv/`:
  ```powershell
  ./scripts/setup-mpv.ps1
  ```
  *(Nixo automatically searches for `libmpv-2.dll` next to the executable, in `vendor/mpv/`, or in your system `PATH`).*

- **Linux**:
  Install `libmpv2` / `mpv-libs` and GUI build dependencies:
  ```bash
  # Debian / Ubuntu
  sudo apt-get install libgtk-3-dev libxcb-render0-dev libxcb-shape0-dev \
                       libxcb-xfixes0-dev libxkbcommon-dev libwayland-dev libmpv-dev
  ```

- **macOS**:
  ```bash
  brew install mpv
  ```

---

### 2. Running the Player

To run the application:

```bash
cargo run -p nixo-app
```

Or pass a media file directly via the command line:

```bash
cargo run -p nixo-app -- path/to/video.mkv
```

---

### 3. Packaging & Distribution

To build a standalone, optimized redistributable bundle (LTO, strip, embedded DLL):

```powershell
./scripts/package.ps1
```

The output zip file and its SHA256 checksum will be generated inside the `dist/` directory:
- `dist/nixo-v<version>-windows-x86_64.zip`
- `dist/nixo-v<version>-windows-x86_64.zip.sha256`

---

## Keyboard Shortcuts

| Shortcut | Action |
| :--- | :--- |
| `Space` | Play / Pause |
| `Right Arrow` / `Left Arrow` | Seek Forward / Backward 5 seconds |
| `Up Arrow` / `Down Arrow` | Volume Up / Down 5% |
| `M` | Toggle Mute |
| `V` | Toggle Subtitles On / Off |
| `S` | Cycle Subtitle Track |
| `B` | Cycle Audio Track |
| `F` / `F11` / `Double Click` | Toggle Fullscreen |
| `Esc` | Exit Fullscreen |
| `Ctrl + O` | Open file with native dialog |

---

## License

Distributed under the [MIT License](LICENSE).
