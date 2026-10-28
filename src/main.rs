mod app;
mod appearance;
mod command;
mod model;
mod pomodoro;
mod view;

use app::App;
use clap::Parser;
use crossterm::event::{self, Event};
use model::{Countdown, parse_duration};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::{
    io,
    time::{Duration, Instant},
};
use view::CountdownView;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "A TUI countdown and Pomodoro timer with ASCII art"
)]
struct Args {
    #[arg(short, long, default_value = "1m")]
    time: String,
    /// Start a Pomodoro session (25m work, 5m break, 15m rest).
    #[arg(long, conflicts_with = "time")]
    pomo: bool,
}

struct TerminalSession {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
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
    if args.pomo {
        let mut app = App::new(Countdown::new(Duration::ZERO));
        app.start_pomodoro();
        return run(app);
    }
    let Some(duration) = parse_duration(&args.time) else {
        println!(
            "The time format is incorrect or set to 0. Please use a format such as \"--time 1h30m10s\"."
        );
        return Ok(());
    };

    run(App::new(Countdown::new(duration)))
}

fn run(mut app: App) -> Result<(), Box<dyn std::error::Error>> {
    let view = CountdownView::new()?;
    let mut session = TerminalSession::new()?;
    let tick_rate = Duration::from_millis(200);
    let mut last_update = Instant::now();

    loop {
        let now = Instant::now();
        app.tick(now.duration_since(last_update));
        last_update = now;
        session.terminal.draw(|frame| view.render(frame, &app))?;

        if event::poll(tick_rate)? {
            let event = event::read()?;
            // Attribute all time before a command to the previous timer state.
            let now = Instant::now();
            app.tick(now.duration_since(last_update));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pomodoro_startup_and_explicit_time_are_mutually_exclusive() {
        let args = Args::try_parse_from(["coffee_clock", "--pomo"]).unwrap();
        assert!(args.pomo);
        assert!(Args::try_parse_from(["coffee_clock", "--pomo", "--time", "5m"]).is_err());
        let args = Args::try_parse_from(["coffee_clock", "--time", "5m"]).unwrap();
        assert!(!args.pomo);
        assert_eq!(args.time, "5m");
    }
}
