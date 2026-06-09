# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

## [3.1.0] - 2026-06-09
### Changed
- **Renamed project** from `rstartup-tui` to `rstartup` (matching `rfetch`).
- **Strict Modularity**: Split monolithic `src/main.rs` (2091 lines) into cleaner submodules (`src/app/`, `src/ui/`, `src/win32.rs`).
- Moved backup/restore structures to `src/startup/backup_win.rs`.
- All source files are now strictly under 500 lines.
### Fixed
- Fixed TUI mouse selection click interception, resolving mouse selection jitters and allowing direct list item selection on click.
- Fixed Rust 2024 edition compilation error with unsafe extern blocks in `src/startup/backup_win.rs`.

### Changed
- **Renamed project** from `rStartup` / `rStart` to `rStartup-tui`. The GitHub repository, Cargo package name, binary name, and all user-facing labels now use the `-tui` suffix to make the program's role as a terminal user interface explicit (matching `rTemplate-tui`).
  - Repository: `local76/rStartup` → `local76/rStartup-tui`
  - Crate/binary: `rstart` → `rstartup-tui`
  - Console title: `rStart` → `rStartup-tui`
  - Config file: `%APPDATA%\rStart\config.yaml` → `%APPDATA%\rStartup-tui\config.yaml`
  - Log file: `%APPDATA%\rStart\log.txt` → `%APPDATA%\rStartup-tui\log.txt`
  - Linux package names: `rstart` → `rstartup-tui`

## [3.0.1] - 2026-06-06
### Added
- Added author and maintainer metadata for packaging.

## [3.0.0] - 2026-06-06
### Changed
- Renamed organization to `local76`.
- Renamed executable from `rtem` to `rstart`.
- Reorganized directory structure to group packaging files inside `dist/packages/`.