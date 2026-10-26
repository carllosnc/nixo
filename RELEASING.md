# Release Guide & Procedure — Nixo

This document defines the official, standardized release workflow for **Nixo Media Player**. All contributors and AI agents must adhere strictly to these procedures when preparing, verifying, and publishing releases.

---

## 1. Release Lifecycle & Architecture

The Nixo publishing process follows five sequential phases:

```text
[1. Verification & Tests] ➔ [2. PR & Merge to Main] ➔ [3. Version Bump & Changelog] ➔ [4. Local Packaging] ➔ [5. Git Tag & CI/CD]
```

CI/CD on **GitHub Actions** (`.github/workflows/release.yml`) triggers automatically on any tag matching the semantic pattern `v*` (e.g. `v0.1.0`), compiling and publishing checksummed release bundles for Windows and Linux.

---

## 2. Branching & PR Strategy for Releases

1. **Working Branches**: Release preparation work (code refinement, documentation, landing page updates) should occur on a dedicated branch (e.g., `release/v0.1.0` or `release/vX.Y`).
2. **Merge Requirement**: The branch **MUST** be submitted via Pull Request and merged into `main` **before** creating any release tag.
3. **Never Tag Unmerged Branches**: Official release tags must always point to a commit on `main`. Tagging unmerged feature or release branches is strictly prohibited.

---

## 3. Strict Semantic Versioning (SemVer) & Tag Syntax

The project adheres to [Semantic Versioning (SemVer 2.0.0)](https://semver.org/).

### Tag Format
- **Allowed**: `v<MAJOR>.<MINOR>.<PATCH>` (e.g., `v0.1.0`, `v0.1.1`, `v1.0.0`).
- **Prohibited**: Irregular or shorthand forms such as `v.01`, `v0.1`, `release-0.1`, or `0.1.0`.

> [!IMPORTANT]
> GitHub Actions workflows and Inno Setup installer scripts rely on the exact tag format `vX.Y.Z` to construct file names (`nixo-v0.1.0-windows-x86_64.zip`, `nixo-v0.1.0-windows-x86_64-setup.exe`). Using irregular tag names will break package generation or create inconsistent download URLs.

---

## 4. Pre-Release Quality Gates

All checks must pass with zero errors and zero warnings before proceeding:

### 4.1 Automated Tests
```powershell
cargo test --workspace
```
*Requirement:* All unit and integration tests must pass (`0 failed`).

### 4.2 Strict Clippy Lints
```powershell
cargo clippy --workspace --all-targets -- -D warnings
```
*Requirement:* Zero warnings allowed. All warnings are treated as errors.

### 4.3 Code Formatting
```powershell
cargo fmt --all -- --check
```
*Requirement:* Code must strictly match the Rustfmt standard.

---

## 5. Version Bump & Documentation Standards

When preparing a new release:

1. **`Cargo.toml` (Workspace Root)**:
   Update `version` under `[workspace.package]`:
   ```toml
   [workspace.package]
   version = "0.1.0"
   ```

2. **`CHANGELOG.md`**:
   Document changes grouped under standard [Keep a Changelog](https://keepachangelog.com/) sections:
   - `### Added`: New capabilities and features.
   - `### Changed`: Changes in existing functionality or UI improvements.
   - `### Fixed`: Bug fixes and stability improvements.
   - `### Security`: Vulnerability patches or hardening.

3. **`docs/index.html` (Landing Page)**:
   Ensure the version badge in the footer reflects the new release:
   ```html
   <span class="font-mono-timestamp text-xs ...">v0.1</span>
   ```

---

## 6. Landing Page & Web Media Standardization

Whenever updating demo or showcase videos in `docs/assets/`:

1. **Dual-Source Streaming Requirement**:
   Landing page videos must provide both WebM (VP9) and MP4 (H.264) for maximum compression efficiency and universal browser compatibility:
   ```html
   <video ... poster="assets/video-hero-poster.webp">
     <source src="assets/video-hero-section.webm" type="video/webm">
     <source src="assets/video-hero-section.mp4" type="video/mp4">
     Your browser does not support the video tag.
   </video>
   ```

2. **FFmpeg Encoding Parameters**:
   - **MP4 (H.264)**:
     ```powershell
     ffmpeg -i input.mp4 -c:v libx264 -preset slow -crf 20 -pix_fmt yuv420p -movflags +faststart -an docs/assets/video-hero-section.mp4
     ```
   - **WebM (VP9)**:
     ```powershell
     ffmpeg -i input.mp4 -c:v libvpx-vp9 -crf 30 -b:v 0 -pix_fmt yuv420p -deadline good -cpu-used 2 -row-mt 1 -threads 8 -an docs/assets/video-hero-section.webm
     ```
   - **Poster (WebP)**:
     ```powershell
     ffmpeg -ss 00:00:01 -i input.mp4 -vframes 1 -c:v libwebp -quality 85 docs/assets/video-hero-poster.webp
     ```

---

## 7. Local Packaging Verification (Smoke Test)

Before tagging, verify that build scripts produce valid artifacts:

### 7.1 Portable Bundle (.zip)
```powershell
powershell -ExecutionPolicy Bypass -File ./scripts/package.ps1
```
*Output:* `dist/nixo-vX.Y.Z-windows-x86_64.zip` and `.sha256`.

### 7.2 Windows Installer (.exe)
```powershell
powershell -ExecutionPolicy Bypass -File ./scripts/build-installer.ps1
```
*Output:* `dist/nixo-vX.Y.Z-windows-x86_64-setup.exe` and `.sha256`.

### 7.3 Manual Smoke Test Checklist
- [ ] Application launches cleanly from the unpacked portable directory.
- [ ] Video playback functions with audio and subtitle tracks.
- [ ] OSD controls, volume slider, and seekbar respond without lag.
- [ ] Installer wizard renders brand icon and completes installation cleanly.
- [ ] File association allows opening `.mp4` / `.mkv` files via context menu.

---

## 8. Official Publishing Workflow

Once `main` is updated and verified:

### 8.1 Create Annotated Git Tag
```bash
git tag -a v0.1.0 -m "Nixo v0.1.0 — Initial Release"
```

### 8.2 Push Tag to GitHub
```bash
git push origin v0.1.0
```

---

## 9. GitHub Actions Automated Pipeline & Asset Verification

Pushing the tag triggers `.github/workflows/release.yml`:

1. **Quality Gate (`validate`)**: Runs `cargo fmt`, `cargo clippy`, and `cargo test` on Ubuntu.
2. **Windows Matrix (`release-windows`)**: Builds optimized release binaries with LTO, bundles libmpv, compiles the Inno Setup installer, and uploads Windows artifacts.
3. **Linux Matrix (`release-linux`)**: Builds release binaries and packages `.tar.gz` for Linux x86_64.
4. **Release Verification**:
   Navigate to `https://github.com/carllosnc/nixo/releases/tag/vX.Y.Z` and verify all 6 assets:
   - `nixo-vX.Y.Z-windows-x86_64-setup.exe` + `.sha256`
   - `nixo-vX.Y.Z-windows-x86_64.zip` + `.sha256`
   - `nixo-vX.Y.Z-linux-x86_64.tar.gz` + `.sha256`
5. **Publish**: Edit the GitHub Release notes to paste the relevant entry from `CHANGELOG.md` and publish.
