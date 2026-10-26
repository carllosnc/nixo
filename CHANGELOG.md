# Changelog

All notable changes to the **Nixo Media Player** project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.0] - 2026-10-26

### Added
- **100% Native Rust GUI**: High-performance desktop media player built with `eframe`, `egui`, and `wgpu`. Instant startup, zero webviews, and low memory footprint (< 35 MB RAM idle).
- **libmpv Video Engine**: Dynamic runtime linking via `libloading` with full hardware-accelerated decoding support (`D3D11VA`, `NVDEC`, `VAAPI`, `VideoToolbox`).
- **Synthetic Test Engine**: Built-in 60 FPS GPU fallback backend (`SyntheticBackend`) allowing complete UI and playback state testing without external video libraries.
- **Picture-in-Picture (PiP)**: Compact borderless mini-player with free screen dragging, aspect ratio preservation, and double-click instant restore.
- **Virtualized Playlist Drawer**: Smooth animated drawer capable of rendering 10,000+ media files with $O(1)$ memory usage, natural alphanumeric sorting, and edge scrolling.
- **Real-Time Video Equalizer**: Zero-latency Brightness, Contrast, Saturation, and Gamma adjustments operating in the 32-bit RGBA GPU rendering pipeline.
- **Subtitle & Caption Engine**: Full support for embedded and external subtitle formats (`.ass`, `.ssa`, `.srt`, `.vtt`, `.sub`) with custom font selection, styling, and real-time delay synchronization (`Z` / `X` shortcuts).
- **Procedural Vector UI**: Scalable GPU-rendered control icons, volume bars, seeking sliders, and country/language flags.
- **Seek Preview Thumbnail Cache**: Real-time thumbnail preview card upon seekbar hover with non-blocking background frame extraction.
- **Windows Packaging & Installer**: Automated scripts for generating standalone portable `.zip` bundles and Inno Setup `.exe` installers with file associations and Windows context menu integration.
- **CI/CD Automation**: GitHub Actions workflow generating cross-platform artifacts (Windows `.zip`/`.exe` and Linux `.tar.gz`) with SHA-256 checksums on tag creation.
- **Web Landing Page**: Modern showcase website in `docs/` with dual-source video hero streaming (WebM VP9 + MP4 H.264 faststart fallback) and balanced responsive layouts.

### Changed
- Reorganized section headers and grid breakpoints on the landing page to ensure fluid responsiveness on mobile and tablet viewports.
- Repositioned category icons above card titles in the Supported Media section for optimal visual alignment.

### Fixed
- Addressed awkward line breaks in badge pills (`Lossless Hi-Res`, `Styling & Sync`) and section titles across narrow and tablet screen sizes.
