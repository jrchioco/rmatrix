use std::io;
use std::time::{Duration, Instant};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{self, ClearType},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};
use rand::Rng;

use crate::config::Config;

const BAYBAYIN: [char; 23] = [
    '\u{1700}', '\u{1701}', '\u{1702}', '\u{1703}', '\u{1704}', '\u{1705}',
    '\u{1706}', '\u{1707}', '\u{1708}', '\u{1709}', '\u{170A}', '\u{170B}',
    '\u{170C}', '\u{170D}', '\u{170E}', '\u{170F}', '\u{1710}', '\u{1711}',
    '\u{1712}', '\u{1713}', '\u{1714}', '\u{1715}', '\u{1716}',
];

#[derive(Debug, Clone, Copy, PartialEq)]
enum SettingField {
    Speed,
    Density,
    Fps,
    Bold,
    TrailVariability,
    GlitchFrequency,
    TrailColorR,
    TrailColorG,
    TrailColorB,
    HeadColorR,
    HeadColorG,
    HeadColorB,
}

impl SettingField {
    fn next(self) -> Self {
        match self {
            Self::Speed => Self::Density,
            Self::Density => Self::Fps,
            Self::Fps => Self::Bold,
            Self::Bold => Self::TrailVariability,
            Self::TrailVariability => Self::GlitchFrequency,
            Self::GlitchFrequency => Self::TrailColorR,
            Self::TrailColorR => Self::TrailColorG,
            Self::TrailColorG => Self::TrailColorB,
            Self::TrailColorB => Self::HeadColorR,
            Self::HeadColorR => Self::HeadColorG,
            Self::HeadColorG => Self::HeadColorB,
            Self::HeadColorB => Self::Speed,
        }
    }

    fn prev(self) -> Self {
        match self {
            Self::Speed => Self::HeadColorB,
            Self::Density => Self::Speed,
            Self::Fps => Self::Density,
            Self::Bold => Self::Fps,
            Self::TrailVariability => Self::Bold,
            Self::GlitchFrequency => Self::TrailVariability,
            Self::TrailColorR => Self::GlitchFrequency,
            Self::TrailColorG => Self::TrailColorR,
            Self::TrailColorB => Self::TrailColorG,
            Self::HeadColorR => Self::TrailColorB,
            Self::HeadColorG => Self::HeadColorR,
            Self::HeadColorB => Self::HeadColorG,
        }
    }
}

struct PreviewColumn {
    x: u16,
    y: f32,
    speed: f32,
    length: u16,
    glyphs: Vec<char>,
    head_char: char,
}

impl PreviewColumn {
    fn new(config: &Config, x: u16) -> Self {
        let mut rng = rand::thread_rng();
        let length = (6.0 + rng.gen_range(0.0..1.0) * 8.0) as u16;
        let speed = config.speed * (0.3 + rng.gen_range(0.0..1.0) * 0.7);
        let y = rng.gen_range(0.0..1.0) * 20.0;
        let glyphs: Vec<char> = (0..length)
            .map(|_| BAYBAYIN[rng.gen_range(0..BAYBAYIN.len())])
            .collect();
        let head_char = BAYBAYIN[rng.gen_range(0..BAYBAYIN.len())];
        Self { x, y, speed, length, glyphs, head_char }
    }

    fn advance(&mut self, height: u16) {
        self.y += self.speed;
        if self.y - self.length as f32 > height as f32 {
            let mut rng = rand::thread_rng();
            self.length = (6.0 + rng.gen_range(0.0..1.0) * 8.0) as u16;
            self.y = -(rng.gen_range(0.0..1.0) * 3.0);
            self.glyphs = (0..self.length)
                .map(|_| BAYBAYIN[rng.gen_range(0..BAYBAYIN.len())])
                .collect();
            self.head_char = BAYBAYIN[rng.gen_range(0..BAYBAYIN.len())];
        }
    }
}

struct SettingsState {
    config: Config,
    selected: SettingField,
    preview_columns: Vec<PreviewColumn>,
}

impl SettingsState {
    fn new(config: Config) -> Self {
        let preview_columns: Vec<PreviewColumn> = (0..15)
            .enumerate()
            .map(|(i, _)| PreviewColumn::new(&config, i as u16 * 2))
            .collect();
        Self {
            config,
            selected: SettingField::Speed,
            preview_columns,
        }
    }

    fn adjust(&mut self, delta: i32) {
        let d = delta as f32;
        match self.selected {
            SettingField::Speed => {
                self.config.speed = (self.config.speed + d * 0.1).clamp(0.1, 5.0);
            }
            SettingField::Density => {
                self.config.density = (self.config.density + d * 0.05).clamp(0.0, 1.0);
            }
            SettingField::Fps => {
                let v = self.config.fps as i32 + delta * 5;
                self.config.fps = v.clamp(1, 60) as u32;
            }
            SettingField::Bold => {}
            SettingField::TrailVariability => {
                self.config.trail_variability = (self.config.trail_variability + d * 0.05).clamp(0.0, 1.0);
            }
            SettingField::GlitchFrequency => {
                self.config.glitch_frequency = (self.config.glitch_frequency + d * 0.01).clamp(0.0, 1.0);
            }
            SettingField::TrailColorR => {
                let v = self.config.trail_color.r as i32 + delta * 5;
                self.config.trail_color.r = v.clamp(0, 255) as u8;
            }
            SettingField::TrailColorG => {
                let v = self.config.trail_color.g as i32 + delta * 5;
                self.config.trail_color.g = v.clamp(0, 255) as u8;
            }
            SettingField::TrailColorB => {
                let v = self.config.trail_color.b as i32 + delta * 5;
                self.config.trail_color.b = v.clamp(0, 255) as u8;
            }
            SettingField::HeadColorR => {
                let v = self.config.head_color.r as i32 + delta * 5;
                self.config.head_color.r = v.clamp(0, 255) as u8;
            }
            SettingField::HeadColorG => {
                let v = self.config.head_color.g as i32 + delta * 5;
                self.config.head_color.g = v.clamp(0, 255) as u8;
            }
            SettingField::HeadColorB => {
                let v = self.config.head_color.b as i32 + delta * 5;
                self.config.head_color.b = v.clamp(0, 255) as u8;
            }
        }
    }

    fn toggle_bold(&mut self) {
        self.config.bold = !self.config.bold;
    }

    fn update_preview(&mut self) {
        let height = 12;
        for col in self.preview_columns.iter_mut() {
            col.advance(height);
        }
    }
}

fn dimmed(color: &crate::config::RgbColor, factor: f32) -> Color {
    Color::Rgb(
        (color.r as f32 * factor) as u8,
        (color.g as f32 * factor) as u8,
        (color.b as f32 * factor) as u8,
    )
}

fn head_rgb(config: &Config) -> Color {
    Color::Rgb(config.head_color.r, config.head_color.g, config.head_color.b)
}

fn trail_rgb(config: &Config) -> Color {
    Color::Rgb(config.trail_color.r, config.trail_color.g, config.trail_color.b)
}

fn dim_trail(config: &Config) -> Color {
    dimmed(&config.trail_color, 0.4)
}

fn render_slider(label: &str, value: f32, min: f32, max: f32, selected: bool, config: &Config) -> Line<'static> {
    let width = 20;
    let ratio = (value - min) / (max - min);
    let filled = (ratio * width as f32) as usize;
    let empty = width - filled;

    let filled_bar = "█".repeat(filled);
    let empty_bar = "░".repeat(empty);
    let value_text = format!("{:.1}", value);

    let (pointer, style) = if selected {
        ("▸ ", Style::default().fg(head_rgb(config)).add_modifier(Modifier::BOLD))
    } else {
        ("  ", Style::default().fg(dim_trail(config)))
    };

    let dim = Style::default().fg(dim_trail(config));

    Line::from(vec![
        Span::styled(pointer, style),
        Span::styled(format!("{:16}", label), style),
        Span::styled("[", dim),
        Span::styled(filled_bar, head_rgb(config)),
        Span::styled(empty_bar, dim),
        Span::styled("] ", dim),
        Span::styled(value_text, style),
    ])
}

fn render_slider_pct(label: &str, value: f32, min: f32, max: f32, selected: bool, config: &Config) -> Line<'static> {
    let width = 20;
    let ratio = (value - min) / (max - min);
    let filled = (ratio * width as f32) as usize;
    let empty = width - filled;

    let filled_bar = "█".repeat(filled);
    let empty_bar = "░".repeat(empty);
    let pct = (value * 100.0) as u32;
    let value_text = format!("{}%", pct);

    let (pointer, style) = if selected {
        ("▸ ", Style::default().fg(head_rgb(config)).add_modifier(Modifier::BOLD))
    } else {
        ("  ", Style::default().fg(dim_trail(config)))
    };

    let dim = Style::default().fg(dim_trail(config));

    Line::from(vec![
        Span::styled(pointer, style),
        Span::styled(format!("{:16}", label), style),
        Span::styled("[", dim),
        Span::styled(filled_bar, head_rgb(config)),
        Span::styled(empty_bar, dim),
        Span::styled("] ", dim),
        Span::styled(value_text, style),
    ])
}

fn render_toggle(label: &str, value: bool, selected: bool, config: &Config) -> Line<'static> {
    let (pointer, style) = if selected {
        ("▸ ", Style::default().fg(head_rgb(config)).add_modifier(Modifier::BOLD))
    } else {
        ("  ", Style::default().fg(dim_trail(config)))
    };
    let text = if value { "ON " } else { "OFF" };
    let toggle_style = if selected && value {
        Style::default().fg(head_rgb(config)).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(dim_trail(config))
    };
    Line::from(vec![
        Span::styled(pointer, style),
        Span::styled(format!("{:16}", label), style),
        Span::styled(format!("[{}]", text), toggle_style),
    ])
}

fn render_color_slider(label: &str, value: u8, selected: bool, channel_color: Color, config: &Config) -> Line<'static> {
    let width = 15;
    let ratio = value as f32 / 255.0;
    let filled = (ratio * width as f32) as usize;
    let empty = width - filled;

    let filled_bar = "█".repeat(filled);
    let empty_bar = "░".repeat(empty);

    let (pointer, style) = if selected {
        ("▸ ", Style::default().fg(head_rgb(config)).add_modifier(Modifier::BOLD))
    } else {
        ("  ", Style::default().fg(dim_trail(config)))
    };
    let dim = Style::default().fg(dim_trail(config));

    Line::from(vec![
        Span::styled(pointer, style),
        Span::styled(format!("  {:2}", label), Style::default().fg(channel_color).add_modifier(Modifier::BOLD)),
        Span::styled("[", dim),
        Span::styled(filled_bar, Style::default().fg(channel_color)),
        Span::styled(empty_bar, dim),
        Span::styled("] ", dim),
        Span::styled(format!("{:3}", value), style),
    ])
}

fn render_preview(state: &SettingsState, area: Rect, frame: &mut ratatui::Frame) {
    let block = Block::default()
        .title(" preview ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(dim_trail(&state.config)));
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let w = inner.width as usize;
    let h = inner.height as usize;
    if w == 0 || h == 0 {
        return;
    }

    for y in 0..h {
        let mut cells: Vec<(usize, char, Color)> = Vec::new();

        for col in &state.preview_columns {
            let head_y = col.y as i32;
            let trail_len = col.length as usize;
            let cell_x = col.x as usize;

            for dist in 0..trail_len {
                let draw_y = head_y - dist as i32;
                if draw_y as usize == y && cell_x < w {
                    let ratio = 1.0 - (dist as f32 / trail_len as f32);
                    let intensity = ratio * 0.8 + 0.1;
                    let r = (state.config.trail_color.r as f32 * intensity) as u8;
                    let g = (state.config.trail_color.g as f32 * intensity) as u8;
                    let b = (state.config.trail_color.b as f32 * intensity) as u8;

                    let ch = if dist == 0 {
                        col.head_char
                    } else if dist < col.glyphs.len() {
                        col.glyphs[dist]
                    } else {
                        continue;
                    };
                    cells.push((cell_x, ch, Color::Rgb(r, g, b)));
                }
            }
        }

        cells.sort_by_key(|c| c.0);
        let mut spans: Vec<Span> = Vec::new();
        let mut last_x = 0;
        for (x, ch, color) in cells {
            if x > last_x {
                let pad = x - last_x;
                spans.push(Span::styled(" ".repeat(pad), Style::default().fg(dim_trail(&state.config))));
            }
            spans.push(Span::styled(ch.to_string(), Style::default().fg(color)));
            last_x = x + 1;
        }
        if last_x < w {
            spans.push(Span::styled(" ".repeat(w - last_x), Style::default()));
        }

        let line = Line::from(spans);
        let paragraph = Paragraph::new(vec![line]);
        let row_area = Rect {
            x: inner.x,
            y: inner.y + y as u16,
            width: inner.width,
            height: 1,
        };
        frame.render_widget(paragraph, row_area);
    }
}

pub fn run_settings(config: Config) -> io::Result<Config> {
    let mut state = SettingsState::new(config);

    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::Clear(ClearType::All)
    )?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(50);
    let mut last_tick = Instant::now();

    let result = (|| -> io::Result<Config> {
        loop {
            terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(50),
                    Constraint::Percentage(50),
                ])
                .split(f.size());

            let settings_area = chunks[0];
            let preview_area = chunks[1];

            let settings_block = Block::default()
                .title(" rmatrix settings ")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(dim_trail(&state.config)));
            let inner = settings_block.inner(settings_area);
            f.render_widget(settings_block, settings_area);

            let mut lines: Vec<Line> = vec![];

            lines.push(Line::from(""));

            lines.push(Line::from(vec![
                Span::styled("  Speed", if state.selected == SettingField::Speed {
                    Style::default().fg(head_rgb(&state.config)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(dim_trail(&state.config))
                }),
            ]));
            lines.push(render_slider(
                "",
                state.config.speed,
                0.1,
                5.0,
                state.selected == SettingField::Speed,
                &state.config,
            ));

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  Density", if state.selected == SettingField::Density {
                    Style::default().fg(head_rgb(&state.config)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(dim_trail(&state.config))
                }),
            ]));
            lines.push(render_slider(
                "",
                state.config.density,
                0.0,
                1.0,
                state.selected == SettingField::Density,
                &state.config,
            ));

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  FPS", if state.selected == SettingField::Fps {
                    Style::default().fg(head_rgb(&state.config)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(dim_trail(&state.config))
                }),
            ]));
            lines.push(render_slider(
                "",
                state.config.fps as f32,
                1.0,
                60.0,
                state.selected == SettingField::Fps,
                &state.config,
            ));

            lines.push(Line::from(""));
            lines.push(render_toggle(
                "Bold",
                state.config.bold,
                state.selected == SettingField::Bold,
                &state.config,
            ));

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  Trail variability", if state.selected == SettingField::TrailVariability {
                    Style::default().fg(head_rgb(&state.config)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(dim_trail(&state.config))
                }),
            ]));
            lines.push(render_slider(
                "",
                state.config.trail_variability,
                0.0,
                1.0,
                state.selected == SettingField::TrailVariability,
                &state.config,
            ));

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("  Glitch frequency", if state.selected == SettingField::GlitchFrequency {
                    Style::default().fg(head_rgb(&state.config)).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(dim_trail(&state.config))
                }),
            ]));
            lines.push(render_slider_pct(
                "",
                state.config.glitch_frequency,
                0.0,
                1.0,
                state.selected == SettingField::GlitchFrequency,
                &state.config,
            ));

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(
                    "  Trail Color",
                    Style::default().fg(head_rgb(&state.config)).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(render_color_slider(
                "R",
                state.config.trail_color.r,
                state.selected == SettingField::TrailColorR,
                Color::Red,
                &state.config,
            ));
            lines.push(render_color_slider(
                "G",
                state.config.trail_color.g,
                state.selected == SettingField::TrailColorG,
                Color::Green,
                &state.config,
            ));
            lines.push(render_color_slider(
                "B",
                state.config.trail_color.b,
                state.selected == SettingField::TrailColorB,
                Color::Blue,
                &state.config,
            ));

            let trail_preview_style = Style::default().fg(trail_rgb(&state.config));
            lines.push(Line::from(vec![
                Span::styled("    ", trail_preview_style),
                Span::styled("████████", trail_preview_style),
                Span::styled(" preview", Style::default().fg(dim_trail(&state.config))),
            ]));

            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(
                    "  Head Color",
                    Style::default().fg(head_rgb(&state.config)).add_modifier(Modifier::BOLD),
                ),
            ]));
            lines.push(render_color_slider(
                "R",
                state.config.head_color.r,
                state.selected == SettingField::HeadColorR,
                Color::Red,
                &state.config,
            ));
            lines.push(render_color_slider(
                "G",
                state.config.head_color.g,
                state.selected == SettingField::HeadColorG,
                Color::Green,
                &state.config,
            ));
            lines.push(render_color_slider(
                "B",
                state.config.head_color.b,
                state.selected == SettingField::HeadColorB,
                Color::Blue,
                &state.config,
            ));

            let head_preview_style = Style::default().fg(head_rgb(&state.config));
            lines.push(Line::from(vec![
                Span::styled("    ", head_preview_style),
                Span::styled("████████", head_preview_style),
                Span::styled(" preview", Style::default().fg(dim_trail(&state.config))),
            ]));

            lines.push(Line::from(""));
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled(
                    "  ↑/↓: select  ←/→: adjust  Tab: next  Enter: toggle bold",
                    Style::default().fg(dim_trail(&state.config)),
                ),
            ]));
            lines.push(Line::from(vec![
                Span::styled(
                    "  q/Esc: save & start rain",
                    Style::default().fg(dim_trail(&state.config)),
                ),
            ]));

            let paragraph = Paragraph::new(lines);
            f.render_widget(paragraph, inner);

            render_preview(&state, preview_area, f);
        })?;

        let elapsed = last_tick.elapsed();
        if elapsed >= tick_rate {
            state.update_preview();
            last_tick = Instant::now();
        }

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Esc => {
                        return Ok(state.config);
                    }
                    KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        return Ok(state.config);
                    }
                    KeyCode::Up => {
                        state.selected = state.selected.prev();
                    }
                    KeyCode::Down => {
                        state.selected = state.selected.next();
                    }
                    KeyCode::Left => {
                        state.adjust(-1);
                    }
                    KeyCode::Right => {
                        state.adjust(1);
                    }
                    KeyCode::Tab => {
                        state.selected = state.selected.next();
                    }
                    KeyCode::Enter => {
                        state.toggle_bold();
                    }
                    _ => {}
                }
            }
        }
        }
    })();

    // Restore terminal state
    let mut stdout = io::stdout();
    let _ = execute!(
        stdout,
        cursor::Show,
        terminal::LeaveAlternateScreen
    );
    let _ = terminal::disable_raw_mode();

    result
}
