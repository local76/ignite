# ignite

> A local startup-time dashboard for autostart apps and boot diagnostics.

`ignite` is a single-binary app for inspecting the things that
launch at boot — the Windows Run / RunOnce registry keys, the
Startup folders, the scheduled tasks, the systemd user services on
Linux. It also ships a `doctor` command for verifying the log paths,
registry access, and config file integrity of the local76 ecosystem.

`ignite` is part of the [local76](https://github.com/local76/local76)
ecosystem and depends on [`library`](https://github.com/local76/library)
for its widgets and design system.

---

## Features

- **Autostart inventory.** Lists every program configured to launch
  at boot, with its source (registry Run key, Startup folder,
  scheduled task, cron, systemd, etc.), its file size, and its
  last-modified timestamp.
- **Disable / enable.** Toggle a single entry off and back on.
  Changes are reversible: `ignite restore` puts the registry to its
  previous state.
- **`ignite doctor`.** Audits the local76 ecosystem: verifies
  `%APPDATA%\local76\app\<app>\config.yaml` is writable, that the
  winget SQLite is present, that the screensaver registry is in a
  known state, that the `trance` registry is current.
- **`ignite backup`.** Snapshots the current autostart state to
  `%APPDATA%\local76\app\ignite\backups\<timestamp>.yaml` for
  diff-and-restore.
- **Hot-loop caching.** Static registry keys are cached in memory;
  only the modified-time check hits the disk.

---

## Install

### Windows
- **Standalone**: download `ignite.exe` from the
  [latest release](https://github.com/local76/ignite/releases).

### Linux
- **Debian/Ubuntu**: `sudo dpkg -i ignite.deb` (downloaded from the
  release page)

---

## Usage

```
ignite                     # launch the autostart dashboard
ignite list                # one-shot: print the autostart inventory to stdout
ignite doctor              # run boot-time diagnostics
ignite backup              # snapshot the current autostart state
ignite restore <file>      # restore from a snapshot
ignite disable <name>      # disable a single entry
ignite enable <name>       # re-enable a disabled entry
ignite --version
ignite --help
```

Inside the dashboard:

| Key | Action |
|---|---|
| `↑` / `↓` | Move selection |
| `Space` | Toggle the selected entry on / off |
| `b` | Backup the current state |
| `r` | Refresh |
| `q` | Quit |

---

## Configuration

A YAML config file is auto-generated on first run:

- **Windows**: `%APPDATA%\local76\app\ignite\config.yaml`
- **Linux**: `~/.config/local76/app/ignite/config.yaml`

Backups are stored at
`%APPDATA%\local76\app\ignite\backups\` (Windows) or
`~/.local/share/local76/app/ignite/backups/` (Linux).

---

## Build from source

```pwsh
git clone https://github.com/local76/ignite.git
cd ignite
cargo build --release
```

---

## License

MIT. See [LICENSE.md](LICENSE.md).
