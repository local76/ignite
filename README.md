rStartup

A local terminal-based startup applications manager.

How to Install:
- exe / msi / deb / rpm: Download from the releases page (https://github.com/local76/rStartup/releases)
- winget: winget install local76.rStartup
- aur: yay -S rstartup-bin

## Embedding rcommon screensaver effects (rcommon 4.2+)

As of rcommon 4.2, all 10 r* screensaver effects (rMatrix, rBeams,
rBhop, rFire, rFireflies, rFireworks, rLife, rParty, rPour,
rUnstable) are consolidated into the `rcommon::role::application::scenes`
module. If your `Cargo.toml` enables the `scenes` feature, you can
embed any r* effect into this app's TUI without a separate crate:

```rust
use rcommon::core::screensaver::Screensaver;
use rcommon::core::TerminalCell;
use rcommon::role::application::scenes::matrix::Matrix;

// In a Ratatui draw closure:
let mut effect = Matrix::new();
let mut grid = vec![TerminalCell::default(); cols * rows];
effect.update(std::time::Duration::from_millis(16), cols, rows);
effect.draw(&mut grid, cols, rows);
```

Available types in rcommon 4.2:
- `scenes::matrix::Matrix`
- `scenes::beams::Beams`
- `scenes::bhop::BhopDashboard`
- `scenes::fire::FireEffect`
- `scenes::fireflies::Fireflies`
- `scenes::fireworks::Fireworks`
- `scenes::life::LifeEffect`
- `scenes::party::Party`
- `scenes::pour::Pour`
- `scenes::unstable::Unstable`

To run an effect as a standalone terminal screensaver (own raw-tty
loop, Ctrl-C to exit), use `rcommon::screensaver_runtime::run_main`:

```rust
fn main() {
    rcommon::screensaver_runtime::run_main(
        rcommon::role::application::scenes::matrix::Matrix::new(),
        "rMatrix",
    );
}
```

The `screensaver_runtime` module is gated on the `screensaver-runtime`
feature (default-off) â€” enable it in your Cargo.toml if your app needs
to host a screensaver process directly.

For the design system surface (status bar, toast, markdown viewer,
theme + accent colors, layout guard, 12 canonical TUI effects),
import the design faÃ§ade:

```rust
use rcommon::interface::tui::design::prelude::*;
```
