use std::time::Duration;

use crate::{
    appearance::{COLOR_NAMES, FONT_NAMES, Font, parse_color},
    model::parse_duration,
    pomodoro::Phase,
};

pub const COMMANDS: &[&str] = &[
    "q",
    "quit",
    "r",
    "restart",
    "s",
    "stop",
    "set color",
    "set font",
    "sc",
    "sf",
    "c",
    "continue",
    "countdown",
    "cd",
    "t",
    "time",
    "pomo",
    "worktime",
    "breaktime",
    "resttime",
];

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Quit,
    Restart(Option<Duration>),
    RestartPhase(Phase),
    Pomo,
    PhaseTime(Phase, Duration),
    Stop,
    Continue,
    Countdown(Duration),
    SetColor(ratatui::style::Color),
    SetFont(Font),
    Time,
}

impl Command {
    pub fn parse(input: &str) -> Result<Self, &'static str> {
        let input = input.to_ascii_lowercase();
        let mut words = input.split_whitespace();
        let mut name = words.next().ok_or("Enter a command.")?;
        if name == "set" {
            name = match words.next() {
                Some("color") => "sc",
                Some("font") => "sf",
                _ => return Err("Usage: set color <color> or set font <font>"),
            };
        }
        let argument = words.next();
        if words.next().is_some() {
            return Err("Too many arguments.");
        }
        match (name, argument) {
            ("q" | "quit", None) => Ok(Self::Quit),
            ("r" | "restart", None) => Ok(Self::Restart(None)),
            ("r" | "restart", Some(phase)) if Phase::parse(phase).is_some() => {
                Ok(Self::RestartPhase(Phase::parse(phase).unwrap()))
            }
            ("pomo", None) => Ok(Self::Pomo),
            ("worktime" | "breaktime" | "resttime", Some(time)) => {
                let phase = match name {
                    "worktime" => Phase::Work,
                    "breaktime" => Phase::Break,
                    _ => Phase::Rest,
                };
                parse_duration(time)
                    .map(|duration| Self::PhaseTime(phase, duration))
                    .ok_or("Invalid phase duration. Example: worktime 25m")
            }
            ("worktime" | "breaktime" | "resttime", None) => {
                Err("Provide a positive duration, for example: worktime 25m")
            }
            ("r" | "restart", Some(time)) => parse_duration(time)
                .map(|duration| Self::Restart(Some(duration)))
                .ok_or("Invalid duration. Example: restart 1h30m10s"),
            ("s" | "stop", None) => Ok(Self::Stop),
            ("c" | "continue", None) => Ok(Self::Continue),
            ("countdown" | "cd", Some(time)) => parse_duration(time)
                .map(Self::Countdown)
                .ok_or("Invalid duration. Example: countdown 25m"),
            ("countdown" | "cd", None) => Err("Usage: countdown <duration>, for example: cd 25m"),
            ("sc", Some(color)) => parse_color(color)
                .map(Self::SetColor)
                .ok_or("Invalid color. Use a color name or #RRGGBB / #RGB."),
            ("sf", Some(font)) => Font::parse(font)
                .map(Self::SetFont)
                .ok_or("Unknown font: standard, small, big, slant, mini, banner."),
            ("sc", None) => Err("Usage: set color <color> or sc <color>"),
            ("sf", None) => Err("Usage: set font <font> or sf <font>"),
            ("t" | "time", None) => Ok(Self::Time),
            ("q" | "quit" | "s" | "stop" | "c" | "continue" | "t" | "time" | "pomo", Some(_)) => {
                Err("This command takes no arguments.")
            }
            _ => Err("Unknown command. Press Tab to complete a command."),
        }
    }
}

#[derive(Debug, Default)]
pub struct CommandLine {
    buffer: Vec<char>,
    pub cursor: usize,
    completion: Option<(Vec<String>, usize)>,
}

impl CommandLine {
    pub fn text(&self) -> String {
        self.buffer.iter().collect()
    }

    pub fn insert(&mut self, character: char) {
        self.buffer.insert(self.cursor, character);
        self.cursor += 1;
        self.completion = None;
    }

    pub fn backspace(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
            self.buffer.remove(self.cursor);
        }
        self.completion = None;
    }

    pub fn delete(&mut self) {
        if self.cursor < self.buffer.len() {
            self.buffer.remove(self.cursor);
        }
        self.completion = None;
    }

    pub fn move_left(&mut self) {
        self.cursor = self.cursor.saturating_sub(1);
        self.completion = None;
    }

    pub fn move_right(&mut self) {
        self.cursor = (self.cursor + 1).min(self.buffer.len());
        self.completion = None;
    }

    pub fn home(&mut self) {
        self.cursor = 0;
        self.completion = None;
    }

    pub fn end(&mut self) {
        self.cursor = self.buffer.len();
        self.completion = None;
    }

    pub fn suggestions(&self) -> Vec<String> {
        if let Some((candidates, _)) = &self.completion {
            return candidates.clone();
        }
        let text = self.text();
        if text.is_empty() {
            return Vec::new();
        }
        let normalized = text.to_ascii_lowercase();
        for (prefix, values) in [
            ("set color ", COLOR_NAMES),
            ("sc ", COLOR_NAMES),
            ("set font ", FONT_NAMES),
            ("sf ", FONT_NAMES),
            ("restart ", &["work", "break", "rest"][..]),
            ("r ", &["work", "break", "rest"][..]),
        ] {
            if let Some(argument) = normalized.strip_prefix(prefix) {
                return values
                    .iter()
                    .filter(|value| value.starts_with(&argument.to_ascii_lowercase()))
                    .map(|value| format!("{prefix}{value}"))
                    .collect();
            }
        }
        COMMANDS
            .iter()
            .filter(|name| name.starts_with(&normalized))
            .map(|name| (*name).to_owned())
            .collect()
    }

    pub fn selected_suggestion(&self) -> Option<usize> {
        self.completion.as_ref().map(|(_, index)| *index)
    }

    pub fn complete(&mut self, reverse: bool) {
        if let Some((candidates, index)) = &mut self.completion {
            *index = if reverse {
                (*index + candidates.len() - 1) % candidates.len()
            } else {
                (*index + 1) % candidates.len()
            };
            self.buffer = candidates[*index].chars().collect();
        } else {
            let candidates = self.suggestions();
            if candidates.is_empty() {
                return;
            }
            let index = if reverse { candidates.len() - 1 } else { 0 };
            self.buffer = candidates[index].chars().collect();
            self.completion = Some((candidates, index));
        }
        self.cursor = self.buffer.len();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_aliases_and_durations() {
        for (short, long) in [
            ("q", "quit"),
            ("r", "restart"),
            ("s", "stop"),
            ("c", "continue"),
            ("t", "time"),
        ] {
            assert_eq!(Command::parse(short), Command::parse(long));
        }
        assert_eq!(
            Command::parse("restart 1h2m3s"),
            Ok(Command::Restart(Some(Duration::from_secs(3723))))
        );
        assert_eq!(
            Command::parse(" r 25m "),
            Ok(Command::Restart(Some(Duration::from_secs(1500))))
        );
        assert_eq!(
            Command::parse("cd 5s"),
            Ok(Command::Countdown(Duration::from_secs(5)))
        );
    }

    #[test]
    fn commands_and_completions_ignore_ascii_case() {
        assert_eq!(Command::parse("QUIT"), Ok(Command::Quit));
        assert_eq!(Command::parse("SeT CoLoR #F0A"), Command::parse("sc #f0a"));
        assert_eq!(
            Command::parse("RESTART BREAK"),
            Command::parse("restart break")
        );

        let mut line = CommandLine::default();
        for ch in "SET FONT SL".chars() {
            line.insert(ch);
        }
        assert_eq!(line.suggestions(), vec!["set font slant"]);
        line.complete(false);
        assert_eq!(line.text(), "set font slant");

        let mut line = CommandLine::default();
        for ch in "RE".chars() {
            line.insert(ch);
        }
        assert_eq!(line.suggestions(), vec!["restart", "resttime"]);
    }

    #[test]
    fn rejects_invalid_commands_without_accepting_extra_arguments() {
        for input in [
            "",
            "unknown",
            "q now",
            "r 0s",
            "set",
            "set 1m extra",
            "r +1s",
            "set 1s1m",
            "set 25m",
            "cd",
            "countdown 0s",
            "cd 1m extra",
            "sc",
            "sf",
            "set color",
            "set font",
            "sc #xyzxyz",
            "sf nonexistent",
            "set color red extra",
            "set font small extra",
        ] {
            assert!(Command::parse(input).is_err(), "{input}");
        }
    }

    #[test]
    fn parses_pomodoro_settings_and_phase_restarts() {
        assert_eq!(Command::parse("pomo"), Ok(Command::Pomo));
        for (name, phase) in [
            ("work", Phase::Work),
            ("break", Phase::Break),
            ("rest", Phase::Rest),
        ] {
            assert_eq!(
                Command::parse(&format!("restart {name}")),
                Ok(Command::RestartPhase(phase))
            );
            assert_eq!(
                Command::parse(&format!("{name}time 1m")),
                Ok(Command::PhaseTime(phase, Duration::from_secs(60)))
            );
        }
        for input in [
            "pomo 1m",
            "worktime",
            "breaktime 0s",
            "resttime bad",
            "restart work extra",
        ] {
            assert!(Command::parse(input).is_err(), "{input}");
        }
        let mut line = CommandLine::default();
        for ch in "restart b".chars() {
            line.insert(ch);
        }
        line.complete(false);
        assert_eq!(line.text(), "restart break");
    }

    #[test]
    fn parses_appearance_aliases_and_countdown() {
        for (short, long) in [
            ("sc red", "set color red"),
            ("sf small", "set font small"),
            ("cd 25m", "countdown 25m"),
        ] {
            assert_eq!(Command::parse(short), Command::parse(long));
            assert!(Command::parse(short).is_ok());
        }
        assert_eq!(
            Command::parse("sc #f0a"),
            Ok(Command::SetColor(ratatui::style::Color::Rgb(255, 0, 170)))
        );
        assert_eq!(
            Command::parse("set font SLANT"),
            Ok(Command::SetFont(Font::Slant))
        );
    }

    #[test]
    fn completes_subcommands_and_arguments_and_wraps_candidates() {
        for (input, expected) in [
            ("set f", "set font"),
            ("set color li", "set color lightred"),
            ("sc re", "sc red"),
            ("sf sl", "sf slant"),
            ("set font mi", "set font mini"),
        ] {
            let mut line = CommandLine::default();
            for ch in input.chars() {
                line.insert(ch);
            }
            line.complete(false);
            assert_eq!(line.text(), expected);
        }
        let mut line = CommandLine::default();
        for ch in "sf ".chars() {
            line.insert(ch);
        }
        for _ in 0..FONT_NAMES.len() {
            line.complete(false);
        }
        assert_eq!(line.text(), "sf banner");
        line.complete(false);
        assert_eq!(line.text(), "sf standard");
        line.complete(true);
        assert_eq!(line.text(), "sf banner");
        for input in ["cd 25m", "countdown ", "sc #12abef"] {
            let mut line = CommandLine::default();
            for ch in input.chars() {
                line.insert(ch);
            }
            assert!(line.suggestions().is_empty());
            line.complete(false);
            assert_eq!(line.text(), input);
        }
    }

    #[test]
    fn completion_cycles_and_preserves_duration_arguments() {
        let mut line = CommandLine::default();
        line.insert('r');
        line.complete(false);
        assert_eq!(line.text(), "r");
        line.complete(false);
        assert_eq!(line.text(), "restart");
        line.complete(true);
        assert_eq!(line.text(), "r");
        for ch in " 25m".chars() {
            line.insert(ch);
        }
        line.complete(false);
        assert_eq!(line.text(), "r 25m");
    }

    #[test]
    fn edits_unicode_without_splitting_characters() {
        let mut line = CommandLine::default();
        line.insert('中');
        line.insert('r');
        line.move_left();
        line.backspace();
        assert_eq!(line.text(), "r");
        line.delete();
        assert_eq!(line.text(), "");
    }
}
