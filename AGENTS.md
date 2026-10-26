# AGENTS.md — Nixo Media Player

Technical specifications, operational standards, and architectural guidelines for AI coding agents working in the **Nixo** repository.

---

## 1. Project Overview & Architecture

**Nixo** is a high-performance, cross-platform desktop media player written in **Rust (Edition 2024)**.

### Core Principles
- **100% Native GUI (Zero Webview)**: Direct GPU-accelerated rendering powered by `wgpu` and `egui`. No Electron, no CEF, and no web browser engines. Instant startup time and low memory footprint.
- **Hardware-Accelerated Video Playback**: Backed by `libmpv` with hardware decoding (`D3D11VA`, `NVDEC`, `VAAPI`, `VideoToolbox`) and A/V synchronization.
- **Runtime Dynamic Linking**: Dynamically loads `libmpv` at runtime via `libloading`. No C headers or static `.lib` files are required at compile time.
- **Fallback Synthetic Engine**: Ships with a built-in 60 FPS GPU test signal generator (`SyntheticBackend`). UI rendering, texture pipelines, and seekbars can be fully developed and tested without requiring native video libraries.

---

## 2. Cargo Workspace Structure

The project is structured as a modular Cargo Workspace:

```text
nixo/
├── Cargo.toml                 # Root workspace manifest & release profiles
├── CHANGELOG.md               # Version history following Keep a Changelog standard
├── RELEASING.md               # Standardized release workflow, SemVer & CI/CD guide
├── assets/                    # Application icons (ICO, PNG)
├── crates/
│   ├── nixo-core/             # Pure domain models, playback states, commands, and MediaBackend trait
│   ├── nixo-engine/           # Dynamic libmpv integration and 60 FPS synthetic test engine
│   │   └── src/
│   │       ├── lib.rs         # Backend factory & fallback orchestration
│   │       ├── synthetic.rs   # 60 FPS GPU synthetic test generator
│   │       └── mpv/           # Modular libmpv backend (mod.rs, ffi.rs, properties.rs, preview.rs)
│   └── nixo-app/              # Native GUI application (eframe/wgpu/egui window, OSD, keybindings)
│       └── src/
│           ├── main.rs        # Application entry point & window initialization
│           ├── app.rs         # Core NixoApp state, lifecycle, and central video canvas
│           ├── theme.rs       # System font discovery, egui visuals, and Win32 dark titlebar
│           ├── input.rs       # Keyboard shortcuts, mouse hover state machine, and drag-and-drop
│           ├── osd.rs         # Bottom OSD playback control bar and seekbar orchestration
│           ├── popovers/      # Settings menu: style.rs (design tokens), components/ (unique reusable widgets), tabs.rs, subtitle_style.rs
│           ├── widgets.rs     # Reusable UI components (sliders, seek preview card, tooltips)
│           ├── icons.rs       # Procedural GPU vector control icons and ghost buttons
│           ├── flags.rs       # Procedural GPU vector country/language flags
│           └── settings.rs    # Persistent user settings (JSON with background debounce)
├── scripts/
│   ├── setup-mpv.ps1          # Automatic libmpv setup script for Windows
│   ├── package.ps1            # Standalone redistributable bundler (.zip + SHA256)
│   └── generate-icon.ps1      # Procedural icon generator
└── vendor/                    # Local binary storage (e.g., libmpv-2.dll on Windows)
```

### Module Responsibilities & Boundary Rules
1. **`crates/nixo-core`**:
   - Contains pure domain logic: `MediaBackend` trait, `PlayerCommand`, `PlayerStatus`, `PlaybackState`, and `VideoFrame`.
   - **MUST NOT** depend on `egui`, `wgpu`, or video decoder libraries.
   - Keep this crate lightweight and free of UI or low-level FFI dependencies.
2. **`crates/nixo-engine`**:
   - Implements the `MediaBackend` trait via `MpvBackend` (`src/mpv/`) and `SyntheticBackend` (`src/synthetic.rs`).
   - Encapsulates all unsafe FFI bindings (`src/mpv/ffi.rs`), property accessors (`src/mpv/properties.rs`), and background seek thumbnail generation (`src/mpv/preview.rs`).
   - Must handle the absence of `libmpv` gracefully by falling back to `SyntheticBackend`.
3. **`crates/nixo-app`**:
   - Manages application lifecycle (`eframe`), UI state, texture uploads to GPU (`egui::ColorImage`), and user interactions (drag-and-drop, hotkeys, OSD).
   - Organized into focused submodules (`app`, `theme`, `input`, `osd`, `popovers`, `widgets`, `icons`, `flags`, `settings`) so each file stays within the **200–500 lines** target range for optimal AI agent context efficiency.
   - Must never execute blocking operations on the main render thread.

---

## 3. Essential Commands for Agents

Run these commands to validate changes:

### Build & Check
```bash
# Check all workspace targets
cargo check --workspace

# Build entire workspace
cargo build --workspace

# Build optimized release binaries
cargo build --release -p nixo-app
```

### Automated Testing
```bash
# Run all workspace unit and integration tests
cargo test --workspace
```

### Linting & Formatting (Strict CI Requirements)
CI enforces zero warnings on clippy and rustfmt. Always run these before concluding work:
```bash
# Check formatting
cargo fmt --all -- --check

# Format code
cargo fmt --all

# Run Clippy with warnings treated as errors
cargo clippy --workspace --all-targets -- -D warnings
```

### Running the Application
```bash
# Run the GUI application
cargo run -p nixo-app

# Run and immediately open a media file
cargo run -p nixo-app -- "path/to/video.mp4"
```

### Setup & Utility Scripts (Windows PowerShell)
```powershell
# Download and install libmpv-2.dll locally
powershell -ExecutionPolicy Bypass -File ./scripts/setup-mpv.ps1

# Create a redistributable package (.zip with checksum)
powershell -ExecutionPolicy Bypass -File ./scripts/package.ps1
```

---

## 4. Coding Standards & Rust Guidelines

- **Rust Edition**: `2024`.
- **Workspace Dependencies**: Declare shared dependencies in the root `Cargo.toml` under `[workspace.dependencies]` and reference them in member crates with `{ workspace = true }`.
- **Error Handling**:
  - Use `thiserror` for library error definitions (`NixoError`).
  - Do not use `unwrap()` or `expect()` in runtime code or within the UI render loop. Handle errors gracefully and report them via logs or status displays.
- **Logging**:
  - Use the `tracing` crate (`info!`, `warn!`, `debug!`, `error!`).
  - Avoid raw `println!` calls in library crates and the playback loop.
- **Modularity & File Size (AI Agent Ergonomics — 200~500 Lines Rule)**:
  - Keep individual `.rs` source files between **200 and 500 lines** (hard ceiling of **500 lines**). Smaller entry points or glue modules (`main.rs`, `lib.rs`, `input.rs`) may have fewer than 200 lines when appropriate, but no file should exceed ~500 lines.
  - Keep individual functions concise (**50 to 150 lines** max) so their entire scope, local variables, and borrow lifetimes are visible at once.
  - When adding new UI components, popovers, or engine subsystems, split them into dedicated submodules with a single clear responsibility rather than growing existing files past 500 lines.
- **Unsafe Code & FFI**:
  - `unsafe` blocks are restricted to FFI calls in `crates/nixo-engine/src/mpv/` and Win32 window attribute calls in `crates/nixo-app/src/theme.rs`.
  - Every `unsafe` block must be accompanied by an explicit `// SAFETY: <justification>` comment detailing why the call is safe.
  - The workspace warns on `unsafe_op_in_unsafe_fn`.
- **Cross-Platform Compatibility**:
  - Always use `std::path::Path` / `PathBuf` instead of hardcoded `/` or `\` separators.
  - Accommodate dynamic library naming differences across platforms (`libmpv-2.dll` on Windows, `libmpv.so` on Linux, `libmpv.dylib` on macOS).

---

## 5. Workflow for Adding New Features

When extending the player, follow this layered approach:
1. **Domain (`nixo-core`)**: Declare new commands in `PlayerCommand` or status attributes in `PlayerStatus`.
2. **Engine (`nixo-engine`)**: Implement the logic in both `MpvBackend` (real playback) and `SyntheticBackend` (mock/fallback behavior).
3. **App (`nixo-app`)**: Wire the command into the UI controls (OSD bar, contextual menus, or keyboard shortcuts).
4. **Validation**: Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`.

---

## 6. Release Workflow & Operational Standards

When preparing, testing, or publishing new releases, agents must strictly follow the procedures in [RELEASING.md](RELEASING.md):
- **Branch & PR Strategy**: All release preparations and version bumps must be submitted via Pull Request and merged into `main` before tagging. Never create release tags on unmerged development branches.
- **Semantic Versioning & Tag Syntax**: Tags must strictly follow the `v<MAJOR>.<MINOR>.<PATCH>` format (e.g. `v0.1.0`). Never use shorthand or irregular tags (`v0.1`, `v.01`, `0.1.0`).
- **Changelog Maintenance**: Every user-facing release must be documented in [CHANGELOG.md](CHANGELOG.md) following the [Keep a Changelog](https://keepachangelog.com/) standard.
- **Web Media Standardization**: Demo and showcase video assets under `docs/assets/` must be optimized with dual-source encoding (WebM VP9 + MP4 H.264 faststart + WebP poster) as specified in `RELEASING.md`.
- **Pre-Release Quality Gate**: Ensure `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo fmt --all -- --check` pass with zero errors and zero warnings prior to tagging.

