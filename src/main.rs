mod app;
mod appearance;
mod command;
mod config;
mod model;
mod pomodoro;
mod view;

use app::App;
use clap::Parser;
use crossterm::event::{self, Event};
use model::{Countdown, parse_duration};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io::{self, IsTerminal, Write},
    path::PathBuf,
    time::{Duration, Instant},
};
use view::CountdownView;

#[derive(Parser, Debug)]
#[command(about = "A TUI countdown and Pomodoro timer with ASCII art")]
struct Args {
    #[arg(short, long, default_value = "1m", value_parser = parse_cli_duration)]
    time: Duration,
    /// Start a Pomodoro session (25m work, 5m break, 15m rest).
    #[arg(short, long, conflicts_with = "time")]
    pomo: bool,
    /// Show Information and enjoy a cup of Coffee.
    #[arg(long, conflicts_with = "time", conflicts_with = "pomo")]
    coffee: bool,
    /// Show Version Information.
    #[arg(
        short,
        long,
        conflicts_with = "time",
        conflicts_with = "pomo",
        conflicts_with = "coffee"
    )]
    version: bool,
    /// Use a specific settings file (saved with :save).
    #[arg(long, conflicts_with = "no_config")]
    config: Option<PathBuf>,
    /// Ignore saved settings and disable :save.
    #[arg(long)]
    no_config: bool,
    /// Disable the terminal bell for this session.
    #[arg(long)]
    no_bell: bool,
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

fn parse_cli_duration(input: &str) -> Result<Duration, &'static str> {
    parse_duration(input).ok_or("use a positive duration such as 25m or 1h30m10s")
}

impl TerminalSession {
    fn new() -> io::Result<Self> {
        crossterm::terminal::enable_raw_mode()?;
        let mut stdout = io::stdout();
        if let Err(error) = crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen) {
            let _ = crossterm::terminal::disable_raw_mode();
            return Err(error);
        }
        match Terminal::new(CrosstermBackend::new(stdout)) {
            Ok(terminal) => Ok(Self { terminal }),
            Err(error) => {
                let _ =
                    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen);
                let _ = crossterm::terminal::disable_raw_mode();
                Err(error)
            }
        }
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = self.terminal.show_cursor();
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            self.terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen
        );
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.coffee || args.version {
        information();
        if args.coffee {
            coffee();
        }
        return Ok(());
    }
    let mut app = App::new(Countdown::new(args.time));
    if !args.no_config {
        app.config_path = args.config.or_else(config::default_path);
        if let Some(path) = &app.config_path {
            let settings = config::Settings::load(path)
                .map_err(|error| format!("Cannot load {}: {error}", path.display()))?;
            settings
                .apply(&mut app)
                .map_err(|error| format!("Invalid configuration: {error}"))?;
        }
    }
    if args.no_bell {
        app.bell_enabled = false;
    }
    if args.pomo {
        app.start_pomodoro();
    }
    run(app)
}

fn run(mut app: App) -> Result<(), Box<dyn std::error::Error>> {
    if !io::stdin().is_terminal() || !io::stdout().is_terminal() {
        return Err("Coffee Clock requires an interactive terminal".into());
    }
    let previous_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            io::stdout(),
            crossterm::terminal::LeaveAlternateScreen,
            crossterm::cursor::Show
        );
        previous_hook(info);
    }));
    let view = CountdownView::new()?;
    let mut session = TerminalSession::new()?;
    let tick_rate = Duration::from_millis(200);
    let mut last_update = Instant::now();

    loop {
        let now = Instant::now();
        tick(
            &mut app,
            now.duration_since(last_update),
            session.terminal.backend_mut(),
        )?;
        last_update = now;
        session.terminal.draw(|frame| view.render(frame, &app))?;

        if event::poll(tick_rate)? {
            let event = event::read()?;
            // Attribute all time before a command to the previous timer state.
            let now = Instant::now();
            tick(
                &mut app,
                now.duration_since(last_update),
                session.terminal.backend_mut(),
            )?;
            last_update = now;
            if let Event::Key(key) = event
                && app.handle_key(key)
            {
                break;
            }
        }
    }

    Ok(())
}

fn tick(app: &mut App, elapsed: Duration, output: &mut impl Write) -> io::Result<()> {
    if app.tick(elapsed) && app.bell_enabled {
        output.write_all(b"\x07")?;
        output.flush()?;
    }
    Ok(())
}

fn information() {
    println!("Coffee Clock - {}\nBy Muimi", env!("CARGO_PKG_VERSION"));
}

fn coffee() {
    println!(
        r#"
Here is a cup of coffee. I wish you could enjoy it.


                       (
                        )     (
                 ___...(-------)-....___
             .-""       )    (          ""-.
       .-'``'|-._             )         _.-|
      /  .--.|   `""---...........---""`   |
     /  /    |                             |
     |  |    |                             |
      \  \   |                             |
       `\ `\ |                             |
         `\ `|                             |
         _/ /\                             /
        (__/  \                           /
     _..---""` \                         /`""---.._
  .-'           \                       /          '-.
 :               `-.__             __.-'              :
 :                  ) ""---...---"" (                 :
  '._               `"--...___...--"`              _.'
    \""--..__                              __..--""/
     '._     """----.....______.....----"""     _.'
        `""--..,,_____            _____,,..--""`
                      `"""----"""`

             ASCII Art By Joan G. Stark (Spunk)
"#
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bell_fires_once_on_completion_and_respects_pause_and_mute() {
        let mut app = App::new(Countdown::new(Duration::from_secs(1)));
        let mut output = Vec::new();
        tick(&mut app, Duration::from_millis(500), &mut output).unwrap();
        assert!(output.is_empty());
        app.countdown.stop().unwrap();
        tick(&mut app, Duration::from_secs(10), &mut output).unwrap();
        assert!(output.is_empty());
        app.countdown.resume().unwrap();
        tick(&mut app, Duration::from_millis(500), &mut output).unwrap();
        tick(&mut app, Duration::from_secs(1), &mut output).unwrap();
        assert_eq!(output, b"\x07");
        assert_eq!(app.message.as_deref(), Some("Countdown finished."));
        app.countdown.restart(None);
        app.bell_enabled = false;
        tick(&mut app, Duration::from_secs(1), &mut output).unwrap();
        assert_eq!(output, b"\x07");
    }

    #[test]
    fn delayed_pomodoro_ticks_coalesce_alerts_even_when_returning_to_the_same_phase() {
        let mut app = App::new(Countdown::new(Duration::ZERO));
        app.start_pomodoro();
        let mut output = Vec::new();
        tick(&mut app, Duration::from_secs(7800), &mut output).unwrap();
        assert_eq!(output, b"\x07");
        assert_eq!(app.message.as_deref(), Some("Work started."));
        tick(&mut app, Duration::ZERO, &mut output).unwrap();
        assert_eq!(output, b"\x07");
    }

    #[test]
    fn pomodoro_startup_and_explicit_time_are_mutually_exclusive() {
        let args = Args::try_parse_from(["coffee_clock", "--pomo"]).unwrap();
        assert!(args.pomo);
        assert!(Args::try_parse_from(["coffee_clock", "--pomo", "--time", "5m"]).is_err());
        let args = Args::try_parse_from(["coffee_clock", "--time", "5m"]).unwrap();
        assert!(!args.pomo);
        assert_eq!(args.time, Duration::from_secs(300));
    }
}
