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

- **100% Native GUI (Zero Webview)**: Direct GPU rendering via Vulkan, DirectX 12, and Metal through `wgpu` and `egui`. Lightweight memory footprint, no Electron/CEF, and instantaneous startup.
- **Hardware-Accelerated Video Playback**: Backed by `libmpv` with automatic hardware decoding (`D3D11VA`, `NVDEC`, `VAAPI`, `VideoToolbox`) and pristine A/V synchronization.
- **Picture-in-Picture (PiP / Mini-Player)**: Toggle a compact floating window (`I` or OSD button) with `Always on Top`, free-form window dragging, double-click to restore, and hover controls.
- **Video Equalizer & Image Adjustments**: Fine-tune Brightness, Contrast, Saturation, and Gamma in real-time with smooth sliders and an instant reset button.
- **External Subtitle Drag-and-Drop & Customization**: Drag and drop `.srt`, `.ass`, `.ssa`, `.vtt`, or `.sub` files directly onto playing videos. Full customization for fonts, weights, colors, outlines, shadows, background boxes, and vertical positioning.
- **Animated Virtualized Playlist Drawer**: Smooth slide-out playlist drawer (`L`) featuring $O(1)$ virtualized rendering (`ScrollArea::show_rows`), auto-scroll to the currently playing item, natural alphanumeric sorting, repeat modes (Off, All, One), and continuous autoplay.
- **Interactive Timeline Hover Preview**: Real-time seekbar thumbnail scrubbing powered by an asynchronous background worker with disk caching and proactive black frame detection.
- **Audio & Subtitle Sync Delay**: Adjust A/V synchronization in real-time with +/-0.1s delay steppers (`Z`/`X` for subtitles, `J`/`K` for audio).
- **Responsive On-Screen Display (OSD)**: 4-tier anti-collision control bar, YouTube-style scrim gradient, volume slider, playback speed selector (0.25x - 2.0x), and auto-hiding controls.
- **Native OS Dialogs & Drag-and-Drop**: Load videos, folders, and subtitle files via system file dialogs (`rfd`) or seamless drag-and-drop.
- **Library & Watch History**: Persistent watch progress indicators, automatic resume playback from where you left off, and multi-folder media scanning.
- **Lossless Screenshots**: One-click frame capture saved as PNG and automatically copied to the system clipboard (`P` or `Ctrl + S`).
- **Internationalization (i18n)**: Built-in support for English, Portuguese, and Spanish with procedural GPU vector flags and system locale auto-detection.
- **Fallback Synthetic Engine**: Ships with a built-in 60 FPS GPU test signal generator (`SyntheticBackend`), allowing UI and pipeline development without external dependencies.

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
│   │   └── src/
│   │       ├── lib.rs         # Backend factory & fallback orchestration
│   │       ├── synthetic.rs   # 60 FPS GPU synthetic test generator
│   │       └── mpv/           # Dynamic libmpv bindings, properties, preview worker & thumbnail cache
│   └── nixo-app/              # Native GUI application (wgpu/egui window, OSD controls, shortcuts)
│       └── src/
│           ├── main.rs        # Application entry point & window configuration
│           ├── app.rs         # NixoApp state, lifecycle, and central video canvas
│           ├── mini_player.rs # Picture-in-Picture window management & overlay
│           ├── playlist_drawer.rs # Animated virtualized playlist drawer
│           ├── osd_bar.rs     # Responsive 4-tier bottom playback control bar
│           ├── input.rs       # Keyboard navigation, mouse hover state machine, and drag-and-drop
│           ├── popovers/      # Settings menu, video equalizer, audio, speed & subtitle styling
│           ├── components/    # Reusable widgets (dark capsule tooltips, buttons, empty states)
│           ├── home/          # Media library browser, folder scanner & thumbnail pipeline
│           ├── i18n/          # Compile-time typed translations (EN, PT, ES)
│           └── theme.rs       # Visuals, custom fonts, and Windows 11 black titlebar
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

### Playback & Navigation
| Shortcut | Action |
| :--- | :--- |
| `Space` | Play / Pause |
| `Right Arrow` / `Left Arrow` | Seek Forward / Backward 5 seconds |
| `Up Arrow` / `Down Arrow` | Volume Up / Down 5% |
| `M` | Toggle Mute |
| `L` | Toggle Playlist Drawer |
| `Shift + N` / `Page Down` | Next Track in Playlist |
| `Shift + P` / `Page Up` | Previous Track in Playlist |
| `R` | Cycle Repeat Mode (Off / All / One) |
| `Home` | Return to Home Library |
| `Ctrl + O` | Open file with native dialog |

### Subtitles & Audio
| Shortcut | Action |
| :--- | :--- |
| `V` | Toggle Subtitles On / Off |
| `S` | Cycle Subtitle Track |
| `B` | Cycle Audio Track |
| `Z` / `X` | Subtitle Delay -0.1s / +0.1s |
| `J` / `K` | Audio Delay -0.1s / +0.1s |

### Window & Display
| Shortcut | Action |
| :--- | :--- |
| `I` | Toggle Picture-in-Picture (Mini-Player) |
| `F` / `F11` / `Double Click` | Toggle Fullscreen |
| `Esc` | Exit Fullscreen / Restore Mini-Player / Close Playlist |
| `T` | Toggle Always on Top |
| `A` | Toggle Video Fit / Fill screen |
| `Shift + A` | Snap & Lock window to Video Aspect Ratio |
| `P` / `Ctrl + S` | Take Lossless Screenshot (PNG to clipboard & disk) |
| `Delete` | Remove selected folder or watch history entry (on Home) |

---

## License

Distributed under the [MIT License](LICENSE).
