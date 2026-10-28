use std::time::Duration;

use chrono::Local;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::{
    appearance::Appearance,
    command::{Command, CommandLine},
    model::{Countdown, TimerState},
    pomodoro::{Phase, Pomodoro},
};

#[derive(Debug, Clone, Copy)]
pub enum PendingSwitch {
    Pomodoro,
    Countdown(Duration),
}

impl PendingSwitch {
    pub fn prompt(self) -> &'static str {
        match self {
            Self::Pomodoro => "End current countdown and enter Pomodoro?",
            Self::Countdown(_) => "End current Pomodoro and enter countdown?",
        }
    }
}

pub struct Confirmation {
    pub action: PendingSwitch,
    pub yes: bool,
}

pub struct App {
    pub countdown: Countdown,
    pub pomodoro: Pomodoro,
    pub pomo_mode: bool,
    pub confirmation: Option<Confirmation>,
    pub appearance: Appearance,
    pub command_line: Option<CommandLine>,
    pub message: Option<String>,
    pub message_is_error: bool,
    pub show_time: bool,
    message_remaining: Duration,
}

impl App {
    pub fn new(countdown: Countdown) -> Self {
        Self {
            countdown,
            pomodoro: Pomodoro::default(),
            pomo_mode: false,
            confirmation: None,
            appearance: Appearance::default(),
            command_line: None,
            message: None,
            message_is_error: false,
            show_time: false,
            message_remaining: Duration::ZERO,
        }
    }

    pub fn tick(&mut self, elapsed: Duration) {
        if self.pomo_mode {
            self.pomodoro.tick(elapsed);
        } else {
            self.countdown.tick(elapsed);
        }
        self.message_remaining = self.message_remaining.saturating_sub(elapsed);
        if self.message_remaining.is_zero() {
            self.message = None;
            self.message_is_error = false;
        }
    }

    pub fn display_text(&self) -> String {
        if self.show_time {
            Local::now().format("%H:%M:%S").to_string()
        } else if self.pomo_mode {
            self.pomodoro.display_text()
        } else {
            self.countdown.display_text()
        }
    }

    pub fn start_pomodoro(&mut self) {
        self.pomodoro.restart(Phase::Work);
        self.pomo_mode = true;
        self.show_time = false;
    }

    pub fn status(&self) -> &'static str {
        if self.pomo_mode {
            return self.pomodoro.phase().label();
        }
        match self.countdown.state() {
            TimerState::Running => "Running",
            TimerState::Paused => "Paused",
            TimerState::Ready => "Ready",
            TimerState::Finished => "Finished",
        }
    }

    // Returns true only when an explicit quit command is submitted.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if key.kind == KeyEventKind::Release {
            return false;
        }
        if let Some(confirmation) = &mut self.confirmation {
            match key.code {
                KeyCode::Char('y' | 'Y') => {
                    let action = confirmation.action;
                    self.confirmation = None;
                    self.apply_switch(action);
                }
                KeyCode::Char('n' | 'N') | KeyCode::Esc => self.confirmation = None,
                KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::BackTab => {
                    confirmation.yes = !confirmation.yes
                }
                KeyCode::Enter => {
                    let action = confirmation.action;
                    let yes = confirmation.yes;
                    self.confirmation = None;
                    if yes {
                        self.apply_switch(action);
                    }
                }
                _ => {}
            }
            return false;
        }
        if let Some(line) = &mut self.command_line {
            match key.code {
                KeyCode::Esc => self.command_line = None,
                KeyCode::Enter => {
                    let input = line.text();
                    self.command_line = None;
                    return self.execute(&input);
                }
                KeyCode::Backspace => line.backspace(),
                KeyCode::Delete => line.delete(),
                KeyCode::Left => line.move_left(),
                KeyCode::Right => line.move_right(),
                KeyCode::Home => line.home(),
                KeyCode::End => line.end(),
                KeyCode::Tab | KeyCode::Down => line.complete(false),
                KeyCode::BackTab | KeyCode::Up => line.complete(true),
                KeyCode::Char(character)
                    if !key
                        .modifiers
                        .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
                        && !character.is_control() =>
                {
                    line.insert(character)
                }
                _ => {}
            }
        } else if key.code == KeyCode::Char(':') {
            self.command_line = Some(CommandLine::default());
        }
        false
    }

    fn execute(&mut self, input: &str) -> bool {
        let command = match Command::parse(input) {
            Ok(command) => command,
            Err(error) => {
                self.feedback(Err(error));
                return false;
            }
        };
        let result = match command {
            Command::Quit => return true,
            Command::Pomo => {
                if self.pomo_mode {
                    Ok("Already in Pomodoro mode.")
                } else if !self.countdown.is_finished() {
                    self.confirmation = Some(Confirmation {
                        action: PendingSwitch::Pomodoro,
                        yes: false,
                    });
                    return false;
                } else {
                    self.start_pomodoro();
                    Ok("Pomodoro started.")
                }
            }
            Command::RestartPhase(phase) => {
                if self.pomo_mode {
                    self.pomodoro.restart(phase);
                    self.show_time = false;
                    Ok("Pomodoro phase restarted; cycle reset.")
                } else {
                    Err("Use :pomo before restarting a Pomodoro phase.")
                }
            }
            Command::PhaseTime(phase, duration) => self
                .pomodoro
                .set_duration(phase, duration)
                .map(|()| "Phase duration saved for its next entry."),
            Command::Restart(duration) => {
                if self.pomo_mode {
                    if duration.is_some() {
                        Err("Use restart work, restart break or restart rest in Pomodoro mode.")
                    } else {
                        self.pomodoro.restart(self.pomodoro.phase());
                        self.show_time = false;
                        Ok("Pomodoro phase restarted; cycle reset.")
                    }
                } else {
                    self.countdown.restart(duration);
                    self.show_time = false;
                    Ok("Countdown restarted.")
                }
            }
            Command::Stop => {
                if self.pomo_mode {
                    self.pomodoro.stop().map(|()| "Pomodoro paused.")
                } else {
                    self.countdown.stop().map(|()| "Countdown paused.")
                }
            }
            Command::Continue => {
                if self.pomo_mode {
                    self.pomodoro.resume().map(|()| "Pomodoro resumed.")
                } else {
                    self.countdown.resume().map(|()| "Countdown resumed.")
                }
            }
            Command::Countdown(duration) => {
                if self.pomo_mode {
                    self.confirmation = Some(Confirmation {
                        action: PendingSwitch::Countdown(duration),
                        yes: false,
                    });
                    return false;
                }
                self.countdown
                    .set_duration(duration)
                    .map(|()| "Duration set. Use :continue to start.")
            }
            Command::SetColor(color) => {
                self.appearance.color = color;
                Ok("Display color updated.")
            }
            Command::SetFont(font) => {
                self.appearance.font = font;
                Ok("ASCII font updated.")
            }
            Command::Time => {
                self.show_time = !self.show_time;
                if self.show_time {
                    Ok("Showing local time. Use :time to return to timer.")
                } else if self.pomo_mode {
                    Ok("Showing Pomodoro.")
                } else {
                    Ok("Showing countdown.")
                }
            }
        };
        if result.is_ok()
            && matches!(
                command,
                Command::Restart(_) | Command::Stop | Command::Continue | Command::Countdown(_)
            )
        {
            self.show_time = false;
        }
        self.feedback(result);
        false
    }

    fn apply_switch(&mut self, action: PendingSwitch) {
        match action {
            PendingSwitch::Pomodoro => {
                self.start_pomodoro();
                self.feedback(Ok("Pomodoro started."));
            }
            PendingSwitch::Countdown(duration) => {
                let mut countdown = Countdown::new(duration);
                countdown
                    .stop()
                    .expect("Validated positive countdown duration");
                countdown
                    .set_duration(duration)
                    .expect("New countdown is paused");
                self.countdown = countdown;
                self.pomo_mode = false;
                self.show_time = false;
                self.feedback(Ok("Countdown set. Use :continue to start."));
            }
        }
    }

    fn feedback(&mut self, result: Result<&str, &str>) {
        self.message_remaining = Duration::from_secs(4);
        self.message_is_error = result.is_err();
        self.message = Some(match result {
            Ok(message) | Err(message) => message.to_owned(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        App::new(Countdown::new(Duration::from_secs(60)))
    }

    #[test]
    fn command_mode_requires_colon_and_escape_discards_input() {
        let mut app = app();
        assert!(!app.handle_key(KeyCode::Char('q').into()));
        app.handle_key(KeyCode::Char(':').into());
        app.handle_key(KeyCode::Char('q').into());
        app.handle_key(KeyCode::Esc.into());
        assert!(app.command_line.is_none());
        app.handle_key(KeyCode::Char(':').into());
        for ch in "quit".chars() {
            app.handle_key(KeyCode::Char(ch).into());
        }
        assert!(app.handle_key(KeyCode::Enter.into()));
    }

    #[test]
    fn feedback_expires_while_paused_and_new_feedback_resets_timeout() {
        let mut app = app();
        app.execute("stop");
        app.tick(Duration::from_secs(3));
        assert!(app.message.is_some());
        app.execute("unknown");
        app.tick(Duration::from_secs(3));
        assert!(app.message_is_error);
        assert!(app.message.is_some());
        app.handle_key(KeyCode::Char(':').into());
        assert!(app.message.is_some());
        app.tick(Duration::from_secs(1));
        assert!(app.message.is_none());
        assert!(!app.message_is_error);
        assert_eq!(app.countdown.state(), TimerState::Paused);
    }

    #[test]
    fn arrow_keys_and_tab_cycle_completion_in_both_directions() {
        let mut app = app();
        app.handle_key(KeyCode::Char(':').into());
        app.handle_key(KeyCode::Down.into());
        assert_eq!(app.command_line.as_ref().unwrap().text(), "");
        app.handle_key(KeyCode::Char('s').into());
        for (key, expected) in [
            (KeyCode::Down, "s"),
            (KeyCode::Tab, "stop"),
            (KeyCode::Down, "set color"),
            (KeyCode::Up, "stop"),
            (KeyCode::Up, "s"),
            (KeyCode::Down, "stop"),
        ] {
            app.handle_key(key.into());
            let line = app.command_line.as_ref().unwrap();
            assert_eq!(line.text(), expected);
            assert!(line.selected_suggestion().is_some());
        }
        app.handle_key(KeyCode::Enter.into());
        assert_eq!(app.countdown.state(), TimerState::Paused);
    }

    #[test]
    fn stop_countdown_continue_and_restart_follow_state_rules() {
        let mut app = app();
        app.execute("countdown 2m");
        assert!(app.message_is_error);
        assert_eq!(app.countdown.display_text(), "00:01:00");
        app.execute("stop");
        app.tick(Duration::from_secs(20));
        assert_eq!(app.countdown.display_text(), "00:01:00");
        app.execute("cd 2m");
        assert_eq!(app.countdown.state(), TimerState::Ready);
        app.tick(Duration::from_secs(20));
        assert_eq!(app.countdown.display_text(), "00:02:00");
        app.execute("continue");
        app.tick(Duration::from_secs(30));
        assert_eq!(app.countdown.display_text(), "00:01:30");
        app.execute("restart");
        assert_eq!(app.countdown.display_text(), "00:02:00");
        app.execute("r 5s");
        app.tick(Duration::from_secs(5));
        app.execute("continue");
        assert!(app.message_is_error);
        app.execute("cd 10s");
        assert_eq!(app.countdown.state(), TimerState::Ready);
    }

    #[test]
    fn appearance_commands_preserve_timer_and_display_mode() {
        use crate::appearance::Font;
        use ratatui::style::Color;
        let mut app = app();
        app.execute("time");
        app.execute("set color #12abef");
        assert_eq!(app.appearance.color, Color::Rgb(18, 171, 239));
        app.execute("sf small");
        assert_eq!(app.appearance.font, Font::Small);
        assert!(app.show_time);
        assert_eq!(app.countdown.display_text(), "00:01:00");
        for input in ["sc invalid", "set font missing", "set 25m"] {
            app.execute(input);
            assert!(app.message_is_error);
            assert_eq!(app.appearance.color, Color::Rgb(18, 171, 239));
            assert_eq!(app.appearance.font, Font::Small);
            assert!(app.show_time);
        }
        app.execute("stop");
        app.execute("sc red");
        app.execute("set font big");
        assert_eq!(app.countdown.state(), TimerState::Paused);
        assert_eq!(app.appearance.font, Font::Big);
        assert_eq!(app.appearance.color, Color::Red);
        assert!(!app.message_is_error);
    }

    #[test]
    fn mode_switch_requires_confirmation_and_cancel_preserves_state() {
        let mut app = app();
        app.execute("time");
        app.execute("pomo");
        assert!(app.confirmation.is_some());
        assert!(!app.pomo_mode);
        app.tick(Duration::from_secs(2));
        app.handle_key(KeyCode::Enter.into());
        assert!(app.confirmation.is_none());
        assert!(!app.pomo_mode);
        assert!(app.show_time);
        assert_eq!(app.countdown.display_text(), "00:00:58");
        app.execute("pomo");
        app.handle_key(KeyCode::Char('y').into());
        assert!(app.pomo_mode);
        assert_eq!(app.status(), "Work");
        assert_eq!(app.display_text(), "00:25:00");
        app.execute("countdown 2m");
        app.handle_key(KeyCode::Esc.into());
        assert!(app.pomo_mode);
        assert_eq!(app.display_text(), "00:25:00");
        app.execute("cd 2m");
        app.handle_key(KeyCode::Left.into());
        app.handle_key(KeyCode::Enter.into());
        assert!(!app.pomo_mode);
        assert_eq!(app.status(), "Ready");
        assert_eq!(app.display_text(), "00:02:00");
        app.execute("continue");
        app.tick(Duration::from_secs(120));
        app.execute("pomo");
        assert!(app.pomo_mode);
        assert!(app.confirmation.is_none());
    }

    #[test]
    fn pomodoro_commands_defer_settings_pause_and_toggle_clock() {
        let mut app = app();
        app.start_pomodoro();
        app.execute("worktime 2s");
        app.execute("breaktime 3s");
        app.execute("resttime 4s");
        assert_eq!(app.display_text(), "00:25:00");
        app.execute("restart work");
        assert_eq!(app.display_text(), "00:00:02");
        app.execute("stop");
        app.tick(Duration::from_secs(100));
        assert_eq!(app.display_text(), "00:00:02");
        app.execute("continue");
        app.execute("time");
        app.tick(Duration::from_secs(2));
        app.execute("t");
        assert_eq!(app.status(), "Break");
        assert_eq!(app.display_text(), "00:00:03");
        app.execute("restart rest");
        assert_eq!(app.status(), "Rest");
        assert_eq!(app.display_text(), "00:00:04");
        app.execute("restart break");
        app.tick(Duration::from_secs(3));
        assert_eq!(app.status(), "Work");
        app.tick(Duration::from_secs(2));
        assert_eq!(app.status(), "Break");
        app.execute("restart 1m");
        assert!(app.message_is_error);
        assert_eq!(app.status(), "Break");
    }

    #[test]
    fn time_toggles_preserve_running_paused_and_finished_countdowns() {
        let mut app = app();
        app.execute("time");
        app.tick(Duration::from_secs(10));
        app.execute("t");
        assert!(!app.show_time);
        assert_eq!(app.display_text(), "00:00:50");
        assert_eq!(app.countdown.state(), TimerState::Running);
        assert_eq!(app.message.as_deref(), Some("Showing countdown."));
        app.execute("stop");
        app.execute("t");
        app.tick(Duration::from_secs(10));
        app.execute("time");
        assert!(!app.show_time);
        assert_eq!(app.display_text(), "00:00:50");
        assert_eq!(app.countdown.state(), TimerState::Paused);
        app.execute("continue");
        app.execute("time");
        app.tick(Duration::from_secs(60));
        app.execute("time");
        assert!(!app.show_time);
        assert_eq!(app.display_text(), "00:00:00");
        assert_eq!(app.countdown.state(), TimerState::Finished);
    }

    #[test]
    fn clock_mode_does_not_reset_countdown_and_restart_restores_it() {
        let mut app = app();
        app.execute("time");
        assert!(app.show_time);
        assert_eq!(app.display_text().len(), 8);
        app.tick(Duration::from_secs(10));
        assert_eq!(app.countdown.display_text(), "00:00:50");
        app.execute("restart");
        assert!(!app.show_time);
        assert_eq!(app.countdown.display_text(), "00:01:00");
    }
}
