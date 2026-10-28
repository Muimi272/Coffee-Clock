use figlet_rs::FIGfont;
use ratatui::style::Color;

pub const COLOR_NAMES: &[&str] = &[
    "black",
    "red",
    "green",
    "yellow",
    "blue",
    "magenta",
    "cyan",
    "white",
    "gray",
    "darkgray",
    "lightred",
    "lightgreen",
    "lightyellow",
    "lightblue",
    "lightmagenta",
    "lightcyan",
    "orange",
    "purple",
    "pink",
    "default",
];
pub const FONT_NAMES: &[&str] = &["standard", "small", "big", "slant", "mini", "banner"];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Font {
    #[default]
    Standard,
    Small,
    Big,
    Slant,
    Mini,
    Banner,
}

impl Font {
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "standard" => Some(Self::Standard),
            "small" => Some(Self::Small),
            "big" => Some(Self::Big),
            "slant" => Some(Self::Slant),
            "mini" => Some(Self::Mini),
            "banner" => Some(Self::Banner),
            _ => None,
        }
    }

    pub fn load(self) -> Result<FIGfont, String> {
        let content = match self {
            Self::Standard => return FIGfont::standard(),
            Self::Small => include_str!("../assets/fonts/small.flf"),
            Self::Big => include_str!("../assets/fonts/big.flf"),
            Self::Slant => include_str!("../assets/fonts/slant.flf"),
            Self::Mini => include_str!("../assets/fonts/mini.flf"),
            Self::Banner => include_str!("../assets/fonts/banner.flf"),
        };
        // FIGlet permits negative extension tags; figlet-rs only accepts u32 tags.
        // Exclude those optional glyphs while preserving the upstream font assets.
        let lines: Vec<_> = content.lines().collect();
        let mut header: Vec<_> = lines[0].split_whitespace().map(str::to_owned).collect();
        let height = header[1]
            .parse::<usize>()
            .map_err(|error| error.to_string())?;
        let comments = header[5]
            .parse::<usize>()
            .map_err(|error| error.to_string())?;
        let offset = 1 + comments + 102 * height;
        let extensions: Vec<_> = lines[offset..]
            .chunks(height + 1)
            .filter(|glyph| !glyph[0].trim_start().starts_with('-'))
            .collect();
        header[8] = extensions.len().to_string();
        let mut compatible = vec![header.join(" ")];
        compatible.extend(lines[1..offset].iter().map(|line| (*line).to_owned()));
        compatible.extend(
            extensions
                .into_iter()
                .flatten()
                .map(|line| (*line).to_owned()),
        );
        FIGfont::from_content(&compatible.join("\n"))
    }
}

#[derive(Debug)]
pub struct Appearance {
    pub color: Color,
    pub font: Font,
}

impl Default for Appearance {
    fn default() -> Self {
        Self {
            color: Color::Cyan,
            font: Font::Standard,
        }
    }
}

pub fn parse_color(input: &str) -> Option<Color> {
    let name = input.to_ascii_lowercase();
    let color = match name.as_str() {
        "black" => Color::Black,
        "red" => Color::Red,
        "green" => Color::Green,
        "yellow" => Color::Yellow,
        "blue" => Color::Blue,
        "magenta" => Color::Magenta,
        "cyan" => Color::Cyan,
        "white" => Color::White,
        "gray" | "grey" => Color::Gray,
        "darkgray" | "darkgrey" => Color::DarkGray,
        "lightred" => Color::LightRed,
        "lightgreen" => Color::LightGreen,
        "lightyellow" => Color::LightYellow,
        "lightblue" => Color::LightBlue,
        "lightmagenta" => Color::LightMagenta,
        "lightcyan" => Color::LightCyan,
        "orange" => Color::Rgb(255, 165, 0),
        "purple" => Color::Rgb(128, 0, 128),
        "pink" => Color::Rgb(255, 192, 203),
        "default" => Color::Reset,
        _ => {
            let hex = name.strip_prefix('#').unwrap_or(&name);
            if !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return None;
            }
            let value = u32::from_str_radix(hex, 16).ok()?;
            return match hex.len() {
                3 => Some(Color::Rgb(
                    ((value >> 8) as u8) * 17,
                    (((value >> 4) & 15) as u8) * 17,
                    ((value & 15) as u8) * 17,
                )),
                6 => Some(Color::Rgb(
                    (value >> 16) as u8,
                    (value >> 8) as u8,
                    value as u8,
                )),
                _ => None,
            };
        }
    };
    Some(color)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_names_and_hex_without_accepting_invalid_values() {
        for name in COLOR_NAMES {
            assert!(parse_color(name).is_some(), "{name}");
        }
        assert_eq!(parse_color("RED"), Some(Color::Red));
        for value in ["#12AbEF", "12AbEF"] {
            assert_eq!(parse_color(value), Some(Color::Rgb(18, 171, 239)));
        }
        assert_eq!(parse_color("#f0a"), Some(Color::Rgb(255, 0, 170)));
        for value in [
            "",
            "#",
            "#gggggg",
            "#12345",
            "12345678",
            "transparent",
            "-12345",
        ] {
            assert_eq!(parse_color(value), None);
        }
    }

    #[test]
    fn bundled_fonts_render_all_timer_characters_and_completion_text() {
        let mut figures = Vec::new();
        for name in FONT_NAMES {
            let font = Font::parse(name).unwrap().load().unwrap();
            let figure = font
                .convert("0123456789: Coffee Time!")
                .unwrap()
                .to_string();
            assert!(!figure.trim().is_empty(), "{name}");
            figures.push(figure);
        }
        for (index, figure) in figures.iter().enumerate() {
            for other in figures.iter().skip(index + 1) {
                assert_ne!(figure, other);
            }
        }
    }
}
