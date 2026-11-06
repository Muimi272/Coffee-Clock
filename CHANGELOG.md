# Changelog

## 0.2.1

- Require Rust 1.88 and verify the minimum supported version in CI.
- Update Ratatui and Crossterm, removing the affected `lru` 0.12 release from the dependency graph.
- Audit the lockfile for RustSec advisories and add grouped Dependabot updates for Cargo and GitHub Actions.
- Keep the MSRV toolchain input separate from the Rust setup action version to prevent accidental toolchain upgrades.
- Review breaking Cargo updates individually and defer figlet-rs 1.0.0 because it introduces vulnerable ZIP dependencies and incompatible font APIs.
- Verify the exact packaged executable version and complete documentation before publishing all four platform archives.

## 0.2.0

- Add terminal bell and visible feedback when a countdown finishes or a Pomodoro phase changes. Use `--no-bell` or `:set bell off` to mute it.
- Add `:help`, command history with Ctrl-P / Ctrl-N, and `:save` for appearance, Pomodoro durations, and bell settings.
- Load validated TOML settings from the platform configuration directory or an explicit `--config` path; `--no-config` starts with defaults and disables saving.
- Adapt the clock to smaller windows using smaller fonts or plain digits, keep a compact command line available, and cache FIGlet output between display changes.
- Keep Pomodoro pause state visible and restore the terminal before displaying panic details.
- Reject invalid CLI durations with exit code 2, report noninteractive startup clearly, and read the version from Cargo metadata.
- Validate all four release targets in CI, distinguish Intel and Apple Silicon macOS packages, and include documentation and licenses in archives.
- Fix release-note checkout, use locked builds and explicit targets, and migrate Actions to Node.js 24.

## 0.1.0

- Initial countdown and Pomodoro release with a live clock, command completion, bundled FIGlet fonts, and configurable colors.
