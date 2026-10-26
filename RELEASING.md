# Release Guide & Procedure — Nixo

This document outlines the step-by-step workflow for generating, testing, and publishing an official release of **Nixo Media Player**.

---

## 1. Release Lifecycle Overview

The Nixo publishing process follows four sequential phases:

```text
[1. Technical Verification] ➔ [2. Version Bump] ➔ [3. Local Smoke Test] ➔ [4. Git Tag & CI/CD]
```

CI/CD on **GitHub Actions** (`.github/workflows/release.yml`) triggers automatically on any tag starting with `v*` (e.g. `v0.1.0`), producing signed/checksummed binaries for Windows and Linux.

---

## 2. Pre-Release Checklist

Run all mandatory checks in your terminal to ensure strict CI compliance:

### 2.1 Unit and Integration Tests
```powershell
cargo test --workspace
```
*Criterion:* All tests must pass with `0 failed`.

### 2.2 Strict Clippy Lints
```powershell
cargo clippy --workspace --all-targets -- -D warnings
```
*Criterion:* Zero warnings allowed.

### 2.3 Code Formatting
```powershell
cargo fmt --all -- --check
```
*Criterion:* Code must strictly match the Rustfmt standard.

---

## 3. Version Bump & Documentation

When preparing a new semantic version (`vMAJOR.MINOR.PATCH`):

1. **`Cargo.toml` (Workspace root)**:
   Update the `version` field under `[workspace.package]`:
   ```toml
   [workspace.package]
   version = "0.1.0"
   ```

2. **`docs/index.html` (Landing Page)**:
   Verify that the footer badge (`<footer>`) reflects the release:
   ```html
   <span class="font-mono-timestamp text-xs ...">v0.1</span>
   ```

3. **`CHANGELOG.md`**:
   Document changes grouped under standard *Keep a Changelog* sections:
   - `### Added`: New capabilities and features.
   - `### Changed`: UI/UX enhancements and modified behaviors.
   - `### Fixed`: Bug fixes and stability improvements.

4. **Commit Version Bump**:
   ```bash
   git add Cargo.toml Cargo.lock docs/index.html CHANGELOG.md
   git commit -m "chore(release): bump version to vX.Y.Z"
   git push origin main
   ```

---

## 4. Local Packaging Verification (Smoke Test on Windows)

Before pushing a public tag, generate and test release artifacts locally:

### 4.1 Generate Portable Bundle (.zip)
```powershell
powershell -ExecutionPolicy Bypass -File ./scripts/package.ps1
```
*Generates:* `dist/nixo-vX.Y.Z-windows-x86_64.zip` and `.sha256`.

### 4.2 Generate Windows Installer (.exe)
```powershell
powershell -ExecutionPolicy Bypass -File ./scripts/build-installer.ps1
```
*Generates:* `dist/nixo-vX.Y.Z-windows-x86_64-setup.exe` and `.sha256`.

### 4.3 Manual Installer Checklist
- [ ] Green branding banner renders properly in the installer wizard.
- [ ] Installation completes without requiring Administrator privileges (`AppData\Local\Programs\Nixo`).
- [ ] Start Menu and Desktop shortcuts launch the application.
- [ ] Right-clicking a `.mp4` / `.mkv` file displays Nixo in the Windows **"Open with..."** context menu.
- [ ] Uninstaller cleanly removes all application files and Registry keys.

---

## 5. Official Publishing (Git Tag & CI/CD)

Once `main` is clean, tested, and pushed:

### 5.1 Create Annotated Git Tag
```bash
git tag -a v0.1.0 -m "Nixo v0.1.0 — Initial Release"
```

### 5.2 Push Tag to GitHub
```bash
git push origin v0.1.0
```

---

## 6. Post-Release on GitHub Actions

Pushing the tag triggers the **Release** workflow automatically:

1. **Monitor Pipeline**:
   Visit `https://github.com/carllosnc/nixo/actions` and verify:
   - `release-windows`: Builds optimized release with LTO, creates portable `.zip` and Inno Setup `.exe`.
   - `release-linux`: Builds for Linux x86_64 and creates `.tar.gz`.

2. **Verify Release Assets on GitHub**:
   Navigate to `https://github.com/carllosnc/nixo/releases/tag/v0.1.0`:
   - Confirm all 6 assets are attached:
     - `nixo-vX.Y.Z-windows-x86_64-setup.exe`
     - `nixo-vX.Y.Z-windows-x86_64-setup.exe.sha256`
     - `nixo-vX.Y.Z-windows-x86_64.zip`
     - `nixo-vX.Y.Z-windows-x86_64.zip.sha256`
     - `nixo-vX.Y.Z-linux-x86_64.tar.gz`
     - `nixo-vX.Y.Z-linux-x86_64.tar.gz.sha256`
   - Click **Edit release** to paste curated release notes and publish.
