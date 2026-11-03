use figlet_rs::FIGfont;
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use std::cell::RefCell;

use crate::{
    app::App,
    appearance::{FONT_NAMES, Font},
    command::CommandLine,
};

pub struct CountdownView {
    fonts: Vec<(Font, FIGfont)>,
    art_cache: RefCell<ArtCache>,
}

#[derive(Default)]
struct ArtCache {
    text: String,
    figures: Vec<(Font, String, usize, usize)>,
}

const HELP: &str = "COMMANDS (Esc closes, arrows scroll)\n\
:stop / :s             Pause\n\
:continue / :c         Start or resume\n\
:restart / :r          Restart current timer/phase\n\
:restart 25m           Restart countdown with new time\n\
:countdown 25m / :cd   Set countdown, then :continue\n\
:pomo                  Enter Pomodoro mode\n\
:worktime 25m          Next work duration\n\
:breaktime 5m          Next short break duration\n\
:resttime 15m          Next long break duration\n\
:restart work/break/rest  Restart phase, reset cycle\n\
:time / :t             Toggle local clock\n\
:set font mini / :sf   Choose font\n\
:set color cyan / :sc  Choose color or #RRGGBB\n\
:set bell on/off       Enable or disable terminal bell\n\
:save                  Save settings for next launch\n\
:help                  Show this help\n\
:quit / :q             Exit\n\
INPUT\n\
Tab / Down: next completion; Shift-Tab / Up: previous\n\
Ctrl-P / Ctrl-N: previous / next command history\n\
Left/Right, Home/End: move cursor\n\
Esc: cancel input; Enter: execute\n\
Timers keep running while help is open.";

impl CountdownView {
    pub fn new() -> Result<Self, String> {
        let fonts = FONT_NAMES
            .iter()
            .map(|name| {
                let font =
                    Font::parse(name).ok_or_else(|| format!("Unknown bundled font: {name}"))?;
                font.load()
                    .map(|loaded| (font, loaded))
                    .map_err(|error| format!("Unable to load {name}: {error}"))
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self {
            fonts,
            art_cache: RefCell::default(),
        })
    }

    pub fn render(&self, frame: &mut Frame<'_>, app: &App) {
        let area = frame.size();
        if area.width == 0 || area.height == 0 {
            return;
        }
        let style = Style::default().fg(Color::Cyan);
        if area.height >= 3 {
            frame.render_widget(
                Paragraph::new("Coffee Clock").style(style),
                Rect::new(area.x, area.y, area.width, 1),
            );
        }
        let content = if area.height >= 3 {
            Rect::new(area.x, area.y + 1, area.width, area.height - 2)
        } else {
            Rect::new(area.x, area.y, area.width, 1)
        };
        self.render_timer(frame, content, app);

        if area.height > 1 {
            self.render_status_line(frame, area, app);
        }
        if let Some(line) = &app.command_line {
            self.render_command_line(frame, area, line);
        }
        if let Some(confirmation) = &app.confirmation {
            let width = area.width.min(64);
            let height = area.height.saturating_sub(2).min(5);
            if width >= 3 && height >= 3 {
                let popup = Rect::new(
                    area.x + (area.width - width) / 2,
                    area.y + (area.height - height) / 2,
                    width,
                    height,
                );
                frame.render_widget(Clear, popup);
                let block = Block::default()
                    .borders(Borders::ALL)
                    .title(" Confirm mode switch ")
                    .border_style(Style::default().fg(Color::Yellow));
                let inner = block.inner(popup);
                frame.render_widget(block, popup);
                frame.render_widget(
                    Paragraph::new(confirmation.action.prompt())
                        .wrap(Wrap { trim: true })
                        .style(Style::default().fg(Color::White)),
                    Rect::new(
                        inner.x,
                        inner.y,
                        inner.width,
                        inner.height.saturating_sub(1),
                    ),
                );
                if inner.height > 1 {
                    let options = if confirmation.yes {
                        "[Yes]  No   (Enter / y / n / Esc)"
                    } else {
                        " Yes  [No]  (Enter / y / n / Esc)"
                    };
                    frame.render_widget(
                        Paragraph::new(options).style(Style::default().fg(Color::Yellow)),
                        Rect::new(inner.x, inner.bottom() - 1, inner.width, 1),
                    );
                }
            } else {
                frame.render_widget(Clear, area);
                frame.render_widget(
                    Paragraph::new("Confirm? y/n (Esc cancels)")
                        .style(Style::default().fg(Color::Yellow)),
                    area,
                );
            }
        }
        if app.show_help {
            frame.render_widget(Clear, area);
            frame.render_widget(
                Paragraph::new(HELP)
                    .scroll((app.help_scroll, 0))
                    .style(Style::default().fg(Color::Cyan)),
                area,
            );
        }
    }

    fn render_timer(&self, frame: &mut Frame<'_>, area: Rect, app: &App) {
        let text = app.display_text();
        let mut cache = self.art_cache.borrow_mut();
        if cache.text != text {
            cache.text = text.clone();
            cache.figures.clear();
        }
        let mut figure = None;
        for font in [app.appearance.font, Font::Small, Font::Mini] {
            if !cache.figures.iter().any(|(name, ..)| *name == font) {
                let art = self
                    .fonts
                    .iter()
                    .find(|(name, _)| *name == font)
                    .and_then(|(_, font)| font.convert(&text))
                    .map_or_else(|| text.clone(), |figure| figure.to_string());
                let width = art
                    .lines()
                    .map(|line| ratatui::text::Line::from(line).width())
                    .max()
                    .unwrap_or(0);
                let height = art.lines().count();
                cache.figures.push((font, art, width, height));
            }
            let index = cache
                .figures
                .iter()
                .position(|(name, ..)| *name == font)
                .unwrap();
            let (_, _, width, height) = &cache.figures[index];
            if *width <= usize::from(area.width) && *height <= usize::from(area.height) {
                figure = Some((index, *height as u16));
                break;
            }
        }
        let (art, height) = figure.map_or((text.as_str(), 1), |(index, height)| {
            (cache.figures[index].1.as_str(), height)
        });
        let y = area.y + area.height.saturating_sub(height) / 2;
        frame.render_widget(
            Paragraph::new(art)
                .style(Style::default().fg(app.appearance.color))
                .alignment(Alignment::Center),
            Rect::new(area.x, y, area.width, height.min(area.height)),
        );
    }

    fn render_status_line(&self, frame: &mut Frame<'_>, area: Rect, app: &App) {
        let mode = if app.command_line.is_some() {
            "Command"
        } else {
            "Normal"
        };
        let status = format!("[{}]", app.status());
        let status_width = u16::try_from(status.len())
            .unwrap_or(u16::MAX)
            .min(area.width);
        let y = area.bottom() - 1;
        frame.render_widget(
            Paragraph::new(status).style(Style::default().fg(Color::Cyan)),
            Rect::new(area.x, y, status_width, 1),
        );
        let mode_width = area.width.saturating_sub(status_width.saturating_add(1));
        if mode_width > 0 {
            frame.render_widget(
                Paragraph::new(mode)
                    .alignment(Alignment::Right)
                    .style(Style::default().fg(Color::Cyan)),
                Rect::new(area.right() - mode_width, y, mode_width, 1),
            );
        }
        let title_width = "Coffee Clock".len() as u16;
        let message_width = area.width.saturating_sub(title_width + 1);
        if let Some(message) = &app.message
            && message_width > 0
        {
            let color = if app.message_is_error {
                Color::Red
            } else {
                Color::Green
            };
            frame.render_widget(
                Paragraph::new(message.as_str())
                    .alignment(Alignment::Right)
                    .style(Style::default().fg(color)),
                Rect::new(area.right() - message_width, area.y, message_width, 1),
            );
        }
    }

    fn render_command_line(&self, frame: &mut Frame<'_>, area: Rect, line: &CommandLine) {
        // Keep the title and status row visible, even while the popup is open.
        if area.height < 5 || area.width < 3 {
            let input = Rect::new(area.x, area.bottom() - 1, area.width, 1);
            frame.render_widget(Clear, input);
            self.render_input(frame, input, line);
            return;
        }
        let width = area.width.min(64);
        let x = area.x + (area.width - width) / 2;
        let suggestions = line.suggestions();
        let rows = suggestions
            .len()
            .min(6)
            .min(usize::from(area.height.saturating_sub(5)));
        let popup = Rect::new(x, area.y + 1, width, rows as u16 + 3);
        frame.render_widget(Clear, popup);
        let border = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan));
        let inner = border.inner(popup);
        frame.render_widget(border, popup);
        let input_area = Rect::new(inner.x, inner.y, inner.width, 1);

        self.render_input(frame, input_area, line);
        if rows == 0 {
            return;
        }
        let selected = line.selected_suggestion();
        let start = selected.map_or(0, |index| index.saturating_sub(rows - 1));
        for (row, candidate) in suggestions.iter().skip(start).take(rows).enumerate() {
            let candidate_area = Rect::new(inner.x, inner.y + 1 + row as u16, inner.width, 1);
            let is_selected = selected == Some(start + row);
            let style = if is_selected {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            };
            frame.render_widget(
                Paragraph::new(format!(" {candidate}")).style(style),
                candidate_area,
            );
        }
    }

    fn render_input(&self, frame: &mut Frame<'_>, area: Rect, line: &CommandLine) {
        let text = line.text();
        let before: String = text.chars().take(line.cursor).collect();
        let column = ratatui::text::Line::from(format!(":{before}")).width();
        let last = usize::from(area.width.saturating_sub(1));
        let scroll = u16::try_from(column.saturating_sub(last)).unwrap_or(u16::MAX);
        frame.render_widget(
            Paragraph::new(format!(":{text}"))
                .scroll((0, scroll))
                .style(Style::default().fg(Color::White)),
            area,
        );
        frame.set_cursor(
            area.x + column.saturating_sub(usize::from(scroll)).min(last) as u16,
            area.y,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Countdown;
    use crossterm::event::KeyCode;
    use ratatui::{Terminal, backend::TestBackend};
    use std::time::Duration;

    #[test]
    fn compact_layout_keeps_time_and_commands_visible_after_resizing() {
        let view = CountdownView::new().unwrap();
        let mut app = App::new(Countdown::new(Duration::from_secs(60)));
        app.appearance.font = Font::Banner;
        for (width, height, time_row) in [(12, 3, 1), (12, 2, 0), (8, 1, 0)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal.draw(|frame| view.render(frame, &app)).unwrap();
            assert_eq!(row(&terminal, time_row).trim(), "00:01:00");
        }
        app.tick(Duration::from_secs(1));
        let mut terminal = Terminal::new(TestBackend::new(12, 3)).unwrap();
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        assert_eq!(row(&terminal, 1).trim(), "00:00:59");
        app.handle_key(KeyCode::Char(':').into());
        for ch in "stop".chars() {
            app.handle_key(KeyCode::Char(ch).into());
        }
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        assert_eq!(row(&terminal, 2).trim(), ":stop");
        app.handle_key(KeyCode::Enter.into());
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        assert!(row(&terminal, 2).starts_with("[Paused]"));
    }

    #[test]
    fn help_renders_and_scrolls_without_stopping_the_timer() {
        let view = CountdownView::new().unwrap();
        let mut app = App::new(Countdown::new(Duration::from_secs(60)));
        app.show_help = true;
        let mut terminal = Terminal::new(TestBackend::new(80, 8)).unwrap();
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        assert!(row(&terminal, 0).contains("COMMANDS"));
        app.help_scroll = 15;
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        assert!(row(&terminal, 0).contains(":save"));
        app.handle_key(KeyCode::Esc.into());
        app.tick(Duration::from_secs(1));
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        assert!(row(&terminal, 0).starts_with("Coffee Clock"));
        assert_eq!(app.countdown.display_text(), "00:00:59");
    }

    fn row(terminal: &Terminal<TestBackend>, y: u16) -> String {
        let buffer = terminal.backend().buffer();
        (0..buffer.area.width)
            .map(|x| buffer.get(x, y).symbol())
            .collect()
    }

    #[test]
    fn layout_places_status_and_transient_feedback_at_opposite_corners() {
        let view = CountdownView::new().unwrap();
        let mut app = App::new(Countdown::new(Duration::from_secs(60)));
        let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        assert_eq!(row(&terminal, 0).trim_end(), "Coffee Clock");
        assert!(row(&terminal, 23).starts_with("[Running]"));
        assert!(row(&terminal, 23).ends_with("Normal"));
        app.handle_key(KeyCode::Char(':').into());
        for ch in "stop".chars() {
            app.handle_key(KeyCode::Char(ch).into());
        }
        app.handle_key(KeyCode::Enter.into());
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        let footer = row(&terminal, 23);
        assert!(footer.starts_with("[Paused]"));
        assert!(footer.ends_with("Normal"));
        assert!(row(&terminal, 0).starts_with("Coffee Clock"));
        assert!(row(&terminal, 0).ends_with("Countdown paused."));
        assert!(!footer.contains("Countdown paused."));
        app.tick(Duration::from_secs(4));
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        assert!(row(&terminal, 23).starts_with("[Paused]"));
        assert!(row(&terminal, 23).ends_with("Normal"));
        assert_eq!(row(&terminal, 0).trim_end(), "Coffee Clock");
    }

    #[test]
    fn dropdown_only_appears_after_typing_and_highlights_selection() {
        let view = CountdownView::new().unwrap();
        let mut app = App::new(Countdown::new(Duration::from_secs(60)));
        app.handle_key(KeyCode::Char(':').into());
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        assert!(row(&terminal, 1).contains('┌'));
        assert_eq!(row(&terminal, 2).trim().trim_matches('│').trim(), ":");
        assert!(row(&terminal, 3).contains('└'));
        assert!(row(&terminal, 23).starts_with("[Running]"));
        assert!(row(&terminal, 23).ends_with("Command"));
        app.handle_key(KeyCode::Char('r').into());
        app.handle_key(KeyCode::Down.into());
        app.handle_key(KeyCode::Tab.into());
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        assert_eq!(
            row(&terminal, 2).trim().trim_matches('│').trim(),
            ":restart"
        );
        assert_eq!(row(&terminal, 3).trim().trim_matches('│').trim(), "r");
        assert_eq!(row(&terminal, 4).trim().trim_matches('│').trim(), "restart");
        assert_eq!(
            row(&terminal, 5).trim().trim_matches('│').trim(),
            "resttime"
        );
        assert!(row(&terminal, 6).contains('└'));
        let buffer = terminal.backend().buffer();
        assert_eq!(buffer.get(10, 3).fg, Color::Gray);
        assert_eq!(buffer.get(10, 4).fg, Color::Cyan);
        for y in 1..=6 {
            for x in 8..72 {
                assert_eq!(buffer.get(x, y).bg, Color::Reset);
            }
        }
        app.handle_key(KeyCode::Esc.into());
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        assert!(row(&terminal, 1).trim().is_empty());
        assert!(row(&terminal, 3).trim().is_empty());
        assert!(row(&terminal, 23).ends_with("Normal"));
    }

    #[test]
    fn confirmation_renders_without_background_and_cancel_restores_layout() {
        let view = CountdownView::new().unwrap();
        let mut app = App::new(Countdown::new(Duration::from_secs(60)));
        app.handle_key(KeyCode::Char(':').into());
        for ch in "pomo".chars() {
            app.handle_key(KeyCode::Char(ch).into());
        }
        app.handle_key(KeyCode::Enter.into());
        let mut terminal = Terminal::new(TestBackend::new(100, 24)).unwrap();
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        let screen: String = (0..24).map(|y| row(&terminal, y)).collect();
        assert!(screen.contains("End current countdown and enter Pomodoro?"));
        assert!(screen.contains("[No]"));
        assert!(
            terminal
                .backend()
                .buffer()
                .content
                .iter()
                .all(|cell| cell.bg == Color::Reset)
        );
        for (width, height) in [(1, 1), (3, 5), (30, 6)] {
            let mut small = Terminal::new(TestBackend::new(width, height)).unwrap();
            small.draw(|frame| view.render(frame, &app)).unwrap();
        }
        app.handle_key(KeyCode::Char('n').into());
        assert!(!app.pomo_mode);
        app.start_pomodoro();
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        assert!(row(&terminal, 23).starts_with("[Work | Running]"));
        app.tick(Duration::from_secs(1500));
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        assert!(row(&terminal, 23).starts_with("[Break | Running]"));
        app.tick(Duration::from_secs(5700));
        terminal.draw(|frame| view.render(frame, &app)).unwrap();
        assert!(row(&terminal, 23).starts_with("[Rest | Running]"));
    }

    #[test]
    fn pomodoro_pause_stays_visible_after_feedback_expires_in_every_phase() {
        use crate::pomodoro::Phase;

        let view = CountdownView::new().unwrap();
        let mut app = App::new(Countdown::new(Duration::ZERO));
        app.start_pomodoro();
        let mut terminal = Terminal::new(TestBackend::new(80, 24)).unwrap();

        for phase in [Phase::Work, Phase::Break, Phase::Rest] {
            app.pomodoro.restart(phase);
            for command in ["stop", "continue"] {
                app.handle_key(KeyCode::Char(':').into());
                for ch in command.chars() {
                    app.handle_key(KeyCode::Char(ch).into());
                }
                app.handle_key(KeyCode::Enter.into());
                app.tick(Duration::from_secs(5));
                assert!(app.message.is_none());
                terminal.draw(|frame| view.render(frame, &app)).unwrap();
                let state = if command == "stop" {
                    "Paused"
                } else {
                    "Running"
                };
                assert!(row(&terminal, 23).starts_with(&format!("[{} | {state}]", phase.label())));
            }
        }
    }

    #[test]
    fn renders_selected_fonts_and_custom_color_without_changing_ui_background() {
        let view = CountdownView::new().unwrap();
        let mut app = App::new(Countdown::new(Duration::from_secs(60)));
        let mut terminal = Terminal::new(TestBackend::new(240, 30)).unwrap();
        let mut screens = Vec::new();
        for name in FONT_NAMES {
            app.handle_key(KeyCode::Char(':').into());
            for ch in format!("sf {name}").chars() {
                app.handle_key(KeyCode::Char(ch).into());
            }
            app.handle_key(KeyCode::Enter.into());
            app.handle_key(KeyCode::Char(':').into());
            for ch in "sc #12abef".chars() {
                app.handle_key(KeyCode::Char(ch).into());
            }
            app.handle_key(KeyCode::Enter.into());
            terminal.draw(|frame| view.render(frame, &app)).unwrap();
            let buffer = terminal.backend().buffer();
            assert!(buffer.content.iter().any(
                |cell| cell.fg == Color::Rgb(18, 171, 239) && !cell.symbol().trim().is_empty()
            ));
            assert!(buffer.content.iter().all(|cell| cell.bg == Color::Reset));
            let screen: String = (1..29).map(|y| row(&terminal, y)).collect();
            assert!(
                !screens.contains(&screen),
                "Font {name} must have its own rendering"
            );
            screens.push(screen);
        }
    }

    #[test]
    fn handles_small_terminals_and_long_input() {
        let view = CountdownView::new().unwrap();
        let mut app = App::new(Countdown::new(Duration::from_secs(60)));
        app.command_line = Some(CommandLine::default());
        for _ in 0..100 {
            app.handle_key(KeyCode::Char('中').into());
        }
        for (width, height) in [(1, 1), (1, 3), (2, 2), (12, 3), (3, 5), (30, 6)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal.draw(|frame| view.render(frame, &app)).unwrap();
        }
    }
}
