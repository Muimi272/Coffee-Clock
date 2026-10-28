# Coffee Clock

Coffee Clock 是一个基于 Rust、Ratatui 和 Crossterm 的终端倒计时与番茄钟。它以大型 FIGlet 艺术字显示计时，支持暂停、循环休息、实时系统时间、颜色和字体切换。颜色与字体资源内嵌在程序中，离线即可运行。

## 环境与启动

需要已安装 Rust 稳定版及 Cargo。克隆项目后，在项目目录运行：

```sh
cargo run
```

默认启动 1 分钟倒计时。也可以指定时长，或直接开始默认番茄钟：

```sh
cargo run -- --time 25m
cargo run -- --time 1h30m10s
cargo run -- --pomo
```

时间由正整数和单位组成，单位依次为小时 `h`、分钟 `m`、秒 `s`，例如 `1h30m10s`、`25m`、`45s`。`--pomo` 与 `--time` 不能同时使用。构建发行版可执行文件：

```sh
cargo build --release
```

## 键盘与命令

应用通过 `:` 打开命令栏；输入后按 Enter 执行，按 Esc 放弃输入。方向键、Home、End、Backspace 和 Delete 可编辑命令；Tab 或 ↓ 选择下一个候选，Shift-Tab 或 ↑ 选择上一个候选。候选列表随输入显示，命令及候选匹配不区分大小写。普通模式下按 `q` 不会退出，需执行 `:q` 或 `:quit`。

| 命令 | 作用 |
| --- | --- |
| `q`、`quit` | 退出程序 |
| `r`、`restart` | 重新开始当前倒计时；番茄钟中重新开始当前阶段 |
| `r 25m`、`restart 25m` | 以指定时长重新开始普通倒计时 |
| `s`、`stop` | 暂停计时并保留剩余时间 |
| `c`、`continue` | 开始或继续计时 |
| `cd 25m`、`countdown 25m` | 设置普通倒计时；设置后使用 `:continue` 开始 |
| `t`、`time` | 在计时与本地 24 小时制时间之间切换显示 |
| `pomo` | 开始番茄钟；替换未完成的倒计时前需确认 |
| `worktime 25m` | 设置下一次工作阶段时长 |
| `breaktime 5m` | 设置下一次短休息时长 |
| `resttime 15m` | 设置下一次长休息时长 |
| `restart work`、`restart break`、`restart rest` | 立即重启指定番茄阶段并重置循环计数 |
| `set color cyan`、`sc #12ABEF` | 设置艺术字颜色 |
| `set font slant`、`sf mini` | 设置艺术字字体 |

命令反馈在标题栏右侧显示，数秒后自动消失。进入命令栏或切换到系统时间不会暂停计时。倒计时结束后显示 `00:00:00`；如需再次计时，请重新开始或设置新时长。

## 番茄钟

番茄钟默认包含 25 分钟工作、5 分钟短休息；完成四个工作阶段后进行 15 分钟长休息，随后开始新一轮。各阶段结束后自动切换并开始计时。阶段时长设置仅在下次进入该阶段时生效。`stop` 与 `continue` 可暂停和恢复当前阶段。

运行时可以用 `:pomo` 从普通倒计时切换到番茄钟，或用 `:countdown 25m` 返回普通倒计时；替换另一种模式时会先请求确认。确认框默认选择 No；使用左右键或 Tab 切换选择，按 Enter 提交，也可按 `y` 确认、`n` 或 Esc 取消。等待确认不会暂停当前计时。

## 外观

内置艺术字字体为 `standard`（默认）、`small`、`big`、`slant`、`mini` 和 `banner`。支持常见终端颜色名称（如 `cyan`、`orange`、`purple`、`pink`）以及 `#RRGGBB`、`#RGB` 格式的 HEX 颜色。颜色名和字体名不区分大小写；颜色效果取决于终端的色彩支持。宽体艺术字需要更宽的终端窗口。外观设置只对当前程序运行有效。

字体资源源自 FIGlet 2.2.5 的 `cmatsuoka/figlet` 字体，并随项目分发；许可信息见 [`assets/fonts/LICENSE`](assets/fonts/LICENSE)。字体内容内嵌在可执行文件中，无需运行时下载。

## 开发与验证

```sh
cargo fmt --check
cargo test
cargo clippy --all-targets -- -D warnings
```

## 项目结构

- `src/main.rs`：命令行参数、终端生命周期及事件循环。
- `src/app.rs`：应用状态、键盘事件和命令执行。
- `src/command.rs`：命令解析、输入编辑和自动补全。
- `src/model.rs`：倒计时状态及时间格式解析。
- `src/pomodoro.rs`：工作与休息阶段的循环计时。
- `src/appearance.rs`：颜色解析、字体及外观配置。
- `src/view.rs`：终端界面与艺术字布局。
- `assets/fonts/`：内置 FIGlet 字体资源。
