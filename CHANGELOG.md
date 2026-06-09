# Changelog

All notable changes to this project will be documented in this file.

## [2026.06.09] - 2026-06-09

### Renamed
- **Project rename**: `ignite` was previously `rStartup` (also previously `rstart` in some internal references). The Cargo package name, binary name, file paths, AppData paths, registry keys, and docs are now lowercase `ignite`. Behavior and features are unchanged.

### Fixed
- **Banner / clap name**: the CLI banner and the clap binary name now print `ignite` (they previously printed the legacy `rstart` / `rstartup` strings).

### Changed
- README rewritten in the new register: startup dashboard feature list, install matrix, CLI flags, configuration, build instructions, license.
- Drop the legacy "r*" and "Local freedom" branding throughout.
- Drop the per-repo `rApps` umbrella and `build_all.ps1` from this repo; build orchestration lives in [`toolkit`](https://github.com/local76/toolkit).

## Unreleased

## [3.1.0] - 2026-06-09
### Changed
- **Renamed project** from `ignite-tui` to `ignite` (matching `helm`).
- **Strict Modularity**: Split monolithic `src/main.rs` (2091 lines) into cleaner submodules (`src/app/`, `src/ui/`, `src/win32.rs`).
- Moved backup/restore structures to `src/startup/backup_win.rs`.
- All source files are now strictly under 500 lines.
### Fixed
- Fixed TUI mouse selection click interception, resolving mouse selection jitters and allowing direct list item selection on click.
- Fixed Rust 2024 edition compilation error with unsafe extern blocks in `src/startup/backup_win.rs`.

### Changed
- **Renamed project** from `ignite` / `ignite` to `ignite-tui`. The GitHub repository, Cargo package name, binary name, and all user-facing labels now use the `-tui` suffix to make the program's role as a terminal user interface explicit (matching `template-tui`).
  - Repository: `local76/ignite` → `local76/ignite-tui`
  - Crate/binary: `ignite` → `ignite-tui`
  - Console title: `ignite` → `ignite-tui`
  - Config file: `%APPDATA%\ignite\config.yaml` → `%APPDATA%\ignite-tui\config.yaml`
  - Log file: `%APPDATA%\ignite\log.txt` → `%APPDATA%\ignite-tui\log.txt`
  - Linux package names: `ignite` → `ignite-tui`

## [3.0.1] - 2026-06-06
### Added
- Added author and maintainer metadata for packaging.

## [3.0.0] - 2026-06-06
### Changed
- Renamed organization to `local76`.
- Renamed executable from `rtem` to `ignite`.
- Reorganized directory structure to group packaging files inside `dist/packages/`.