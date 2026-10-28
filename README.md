# Coffee Clock

Coffee Clock is a terminal countdown timer and Pomodoro app built with Rust, Ratatui, and Crossterm. It displays time in large FIGlet art and supports pausing, automatic work and break cycles, a live system clock, and customizable colors and fonts. Fonts are bundled into the executable, so no runtime downloads are needed.

[简体中文文档](docs/README.zh-CN.md)

## Requirements and Setup

Install the stable Rust toolchain and Cargo. From the project directory, run:

```sh
cargo run
```

By default, the app starts a one-minute countdown. You can specify a duration or start a Pomodoro session:

```sh
cargo run -- --time 25m
cargo run -- --time 1h30m10s
cargo run -- --pomo
```

Durations use positive integer values with units in hour, minute, second order: `h`, `m`, and `s`. Examples include `1h30m10s`, `25m`, and `45s`. The `--pomo` and `--time` options are mutually exclusive. Build a release executable with:

```sh
cargo build --release
```

## Keyboard and Commands

Press `:` to open the command bar. Press Enter to execute a command or Esc to cancel. Use the arrow keys, Home, End, Backspace, and Delete to edit input. Tab or ↓ selects the next suggestion; Shift-Tab or ↑ selects the previous one. Suggestions appear as you type. Commands and suggestions are case-insensitive. Pressing `q` in normal mode does not exit; use `:q` or `:quit`.

| Command | Action |
| --- | --- |
| `q`, `quit` | Exit the app |
| `r`, `restart` | Restart the current countdown or Pomodoro phase |
| `r 25m`, `restart 25m` | Restart a regular countdown with the specified duration |
| `s`, `stop` | Pause the timer and retain the remaining time |
| `c`, `continue` | Start or resume the timer |
| `cd 25m`, `countdown 25m` | Set a regular countdown; run `:continue` to start it |
| `t`, `time` | Toggle between the timer and the local 24-hour clock |
| `pomo` | Start Pomodoro mode; replacing an unfinished countdown requires confirmation |
| `worktime 25m` | Set the duration for the next work phase |
| `breaktime 5m` | Set the duration for the next short break |
| `resttime 15m` | Set the duration for the next long break |
| `restart work`, `restart break`, `restart rest` | Restart the specified Pomodoro phase and reset the cycle count |
| `set color cyan`, `sc #12ABEF` | Set the FIGlet display color |
| `set font slant`, `sf mini` | Set the FIGlet font |

Command feedback appears at the right side of the title row and disappears after a few seconds. Opening the command bar or showing the system clock does not pause the timer. A finished countdown displays `00:00:00`; restart it or set a new duration to count again.

## Pomodoro Timer

The default cycle consists of 25 minutes of work followed by a 5-minute short break. After four work phases, the app runs a 15-minute long break and then starts a new cycle. Transitions start automatically. Changes to phase durations take effect the next time that phase begins. Use `stop` and `continue` to pause and resume the current phase.

While the app is running, use `:pomo` to switch from a regular countdown to Pomodoro mode, or `:countdown 25m` to switch back. Replacing the active mode requires confirmation. The confirmation dialog defaults to No; use the Left/Right arrow keys or Tab to change the selection, then press Enter. You can also press `y` to confirm or `n` or Esc to cancel. The timer keeps running while confirmation is pending.

## Appearance

Built-in FIGlet fonts are `standard` (default), `small`, `big`, `slant`, `mini`, and `banner`. Colors can be specified by common terminal color names (such as `cyan`, `orange`, `purple`, and `pink`) or HEX values in `#RRGGBB` and `#RGB` formats. Color and font names are case-insensitive. Color rendering depends on terminal support, and wider fonts may require a wider terminal window. Appearance settings last for the current session only.

Bundled fonts originate from the FIGlet 2.2.5 fonts in the `cmatsuoka/figlet` repository. See [`assets/fonts/LICENSE`](assets/fonts/LICENSE) for their license. Font data is embedded in the executable.

## Development and Validation

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

## Project Structure

- `src/main.rs`: CLI arguments, terminal lifecycle, and event loop.
- `src/app.rs`: Application state, keyboard events, and command execution.
- `src/command.rs`: Command parsing, input editing, and completion.
- `src/model.rs`: Countdown state and duration parsing.
- `src/pomodoro.rs`: Work and break cycle logic.
- `src/appearance.rs`: Color parsing, fonts, and appearance settings.
- `src/view.rs`: Terminal UI and FIGlet layout.
- `assets/fonts/`: Bundled FIGlet fonts.
