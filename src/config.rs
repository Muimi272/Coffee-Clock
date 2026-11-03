use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use directories::ProjectDirs;
use ratatui::style::Color;
use serde::{Deserialize, Serialize};

use crate::{
    app::App,
    appearance::{Font, parse_color},
    model::parse_duration,
    pomodoro::Phase,
};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Settings {
    color: String,
    font: String,
    worktime: String,
    breaktime: String,
    resttime: String,
    bell: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            color: "cyan".into(),
            font: "standard".into(),
            worktime: "25m".into(),
            breaktime: "5m".into(),
            resttime: "15m".into(),
            bell: true,
        }
    }
}

impl Settings {
    pub fn load(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        match fs::read_to_string(path) {
            Ok(text) => Ok(toml::from_str(&text)?),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Self::default()),
            Err(error) => Err(error.into()),
        }
    }

    pub fn apply(&self, app: &mut App) -> Result<(), &'static str> {
        let color = parse_color(&self.color).ok_or("Invalid config color")?;
        let font = Font::parse(&self.font).ok_or("Invalid config font")?;
        let work = parse_duration(&self.worktime).ok_or("Invalid config worktime")?;
        let short = parse_duration(&self.breaktime).ok_or("Invalid config breaktime")?;
        let rest = parse_duration(&self.resttime).ok_or("Invalid config resttime")?;
        app.appearance.color = color;
        app.appearance.font = font;
        app.bell_enabled = self.bell;
        for (phase, duration) in [
            (Phase::Work, work),
            (Phase::Break, short),
            (Phase::Rest, rest),
        ] {
            app.pomodoro.set_duration(phase, duration)?;
        }
        Ok(())
    }

    pub fn capture(app: &App) -> Self {
        let color = match app.appearance.color {
            Color::Reset => "default".to_owned(),
            Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
            color => format!("{color:?}").to_ascii_lowercase(),
        };
        Self {
            color,
            font: app.appearance.font.name().into(),
            worktime: format!("{}s", app.pomodoro.duration(Phase::Work).as_secs()),
            breaktime: format!("{}s", app.pomodoro.duration(Phase::Break).as_secs()),
            resttime: format!("{}s", app.pomodoro.duration(Phase::Rest).as_secs()),
            bell: app.bell_enabled,
        }
    }

    pub fn save(&self, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        fs::create_dir_all(parent)?;
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        file.write_all(toml::to_string_pretty(self)?.as_bytes())?;
        file.as_file().sync_all()?;
        file.persist(path)?;
        Ok(())
    }
}

pub fn default_path() -> Option<PathBuf> {
    ProjectDirs::from("", "", "coffee-clock").map(|dirs| dirs.config_dir().join("config.toml"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Countdown;
    use std::time::Duration;

    #[test]
    fn settings_round_trip_and_replace_an_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/config.toml");
        let mut app = App::new(Countdown::new(Duration::from_secs(60)));
        Settings::default().save(&path).unwrap();
        app.appearance.color = Color::Rgb(18, 171, 239);
        app.appearance.font = Font::Mini;
        app.bell_enabled = false;
        app.pomodoro
            .set_duration(Phase::Work, Duration::from_secs(42))
            .unwrap();
        Settings::capture(&app).save(&path).unwrap();
        let mut restored = App::new(Countdown::new(Duration::from_secs(60)));
        Settings::load(&path).unwrap().apply(&mut restored).unwrap();
        assert_eq!(restored.appearance.color, app.appearance.color);
        assert_eq!(restored.appearance.font, Font::Mini);
        assert!(!restored.bell_enabled);
        assert_eq!(
            restored.pomodoro.duration(Phase::Work),
            Duration::from_secs(42)
        );
    }

    #[test]
    fn partial_config_uses_defaults_and_invalid_values_are_rejected() {
        let mut app = App::new(Countdown::new(Duration::from_secs(60)));
        let settings: Settings = toml::from_str("font = 'mini'").unwrap();
        settings.apply(&mut app).unwrap();
        assert_eq!(app.appearance.font, Font::Mini);
        assert!(app.bell_enabled);
        for text in ["worktime = '0s'", "font = 'missing'", "color = 'missing'"] {
            let invalid: Settings = toml::from_str(text).unwrap();
            assert!(invalid.apply(&mut app).is_err());
            assert_eq!(app.appearance.font, Font::Mini);
        }
        assert!(toml::from_str::<Settings>("unknown = true").is_err());
        assert!(Settings::load(&tempfile::tempdir().unwrap().path().join("absent")).is_ok());
    }
}
