# Changelog

## [2026.6.10] - 2026-06-10

### Changed
- **4.2 Path Modernization**: Updated path imports to align with the `library` 4.2 restructured API (using simplified flat namespaces `apps` and `toolkit`).
- **AppData Directory Realignment**: Moved user configuration, database, and log files into a nested %APPDATA%\local76\app\ignite structure to organize the ecosystem's configuration space.
- **Repository Rename**: Renamed repository and local directory to app-ignite for cleaner ecosystem taxonomy.

## [2026.6.9] - 2026-06-09

### Renamed
- **Project rename**: `ignite` was previously `rStartup` (also previously `rstart` in some internal references). The Cargo package name, binary name, file paths, AppData paths, registry keys, and docs are now lowercase `ignite`. Behavior and features are unchanged.

### Refactored
- **App Blueprint alignment**: Re-architected directory and module tree to standard App layout. Renamed `src/ui/panels.rs` to `src/ui/widgets.rs`. Created `src/backend/` directory, moving `src/startup.rs` to `src/backend/mod.rs` and the Windows, mock, and backup startup modules to `src/backend/startup/`.

### Fixed
- **Banner / clap name**: the CLI banner and the clap binary name now print `ignite` (they previously printed the legacy `rstart` / `rstartup` strings).

### Changed
- README rewritten in the new register: startup dashboard feature list, install matrix, CLI flags, configuration, build instructions, license.
- Drop the legacy "r*" and "Local freedom" branding throughout.
- Drop the per-repo `rApps` umbrella and `build_all.ps1` from this repo; build orchestration lives in [`toolkit`](https://github.com/local76/toolkit).

## Unreleased

## [3.1.0] - 2026-06-09
### Changed
- **Renamed project** from `ignite-App` to `ignite` (matching `helm`).
- **Strict Modularity**: Split monolithic `src/main.rs` (2091 lines) into cleaner submodules (`src/app/`, `src/ui/`, `src/win32.rs`).
- Moved backup/restore structures to `src/startup/backup_win.rs`.
- All source files are now strictly under 500 lines.
### Fixed
- Fixed App mouse selection click interception, resolving mouse selection jitters and allowing direct list item selection on click.
- Fixed Rust 2024 edition compilation error with unsafe extern blocks in `src/startup/backup_win.rs`.

### Changed
- **Renamed project** from `ignite` / `ignite` to `ignite-App`. The GitHub repository, Cargo package name, binary name, and all user-facing labels now use the `-App` suffix to make the program's role as a terminal user interface explicit (matching `template-App`).
  - Repository: `local76/ignite` → `local76/ignite-App`
  - Crate/binary: `ignite` → `ignite-App`
  - Console title: `ignite` → `ignite-App`
  - Config file: `%APPDATA%\ignite\config.yaml` → `%APPDATA%\ignite-App\config.yaml`
  - Log file: `%APPDATA%\ignite\log.txt` → `%APPDATA%\ignite-App\log.txt`
  - Linux package names: `ignite` → `ignite-App`

## [3.0.1] - 2026-06-06
### Added
- Added author and maintainer metadata for packaging.

## [3.0.0] - 2026-06-06
### Changed
- Renamed organization to `local76`.
- Renamed executable from `rtem` to `ignite`.
- Reorganized directory structure to group packaging files inside `dist/packages/`.
