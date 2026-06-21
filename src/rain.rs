use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
    style::{Print, SetForegroundColor},
    terminal::{self, ClearType},
};
use rand::Rng;
use ratatui::style::Color;

use crate::config::Config;

pub const BAYBAYIN: [char; 18] = [
    '\u{1700}', '\u{1701}', '\u{1702}', '\u{1703}', '\u{1704}', '\u{1705}',
    '\u{1706}', '\u{1707}', '\u{1708}', '\u{1709}', '\u{170A}', '\u{170B}',
    '\u{170C}', '\u{170D}', '\u{170E}', '\u{170F}', '\u{1710}', '\u{1711}',
];

pub struct RainColumn {
    pub y: f32,
    pub render_y: i32,
    pub speed: f32,
    pub length: u16,
    pub active: bool,
    pub glyphs: Vec<char>,
    pub head_char: char,
}

impl RainColumn {
    pub fn random(config: &Config, height: f32) -> Self {
        let mut rng = rand::thread_rng();
        let speed = config.speed * (0.5 + rng.gen_range(0.0..1.0) * 1.0);
        let base_length = 8.0_f32;
        let variation = config.trail_variability * 42.0;
        let length = (base_length + rng.gen_range(0.0..1.0) * variation).max(3.0) as u16;
        let y = rng.gen_range(0.0..1.0) * height;
        let glyphs: Vec<char> = (0..length)
            .map(|_| BAYBAYIN[rng.gen_range(0..BAYBAYIN.len())])
            .collect();
        let head_char = BAYBAYIN[rng.gen_range(0..BAYBAYIN.len())];
        Self {
            y,
            render_y: y as i32,
            speed,
            length,
            active: true,
            glyphs,
            head_char,
        }
    }

    pub fn reset(&mut self, config: &Config, height: f32) {
        *self = Self::random(config, height);
        self.y = -(rand::thread_rng().gen_range(0.0..1.0) * 5.0);
        self.render_y = self.y as i32;
    }

    pub fn advance(&mut self, height: u16, config: &Config) {
        let prev_y = self.y;
        self.y += self.speed;
        let prev_render = prev_y as i32;
        let new_render = self.y as i32;
        if prev_render != new_render {
            self.render_y = new_render;
        }
        if self.y - self.length as f32 > height as f32 {
            self.reset(config, height as f32);
        }
    }

    pub fn glitch(&mut self, frequency: f32) {
        let mut rng = rand::thread_rng();
        if rng.gen_range(0.0..1.0) < frequency {
            self.head_char = BAYBAYIN[rng.gen_range(0..BAYBAYIN.len())];
        }
    }
}

fn trail_color(config: &Config, distance_from_head: usize, trail_length: usize) -> Color {
    if distance_from_head == 0 {
        return Color::Rgb(
            config.head_color.r,
            config.head_color.g,
            config.head_color.b,
        );
    }
    let ratio = 1.0 - (distance_from_head as f32 / trail_length as f32);
    let base_r = config.trail_color.r as f32;
    let base_g = config.trail_color.g as f32;
    let base_b = config.trail_color.b as f32;
    let intensity = ratio * 0.7 + 0.1;
    Color::Rgb(
        (base_r * intensity) as u8,
        (base_g * intensity) as u8,
        (base_b * intensity) as u8,
    )
}

pub fn compute_cells(
    columns: &[RainColumn],
    height: usize,
    width: usize,
    config: &Config,
) -> Vec<Vec<(usize, char, Color)>> {
    let mut rows: Vec<Vec<(usize, char, Color)>> = vec![Vec::new(); height];

    for (x, col) in columns.iter().enumerate() {
        if !col.active {
            continue;
        }
        let head_y = col.render_y;
        let trail_len = col.length as usize;
        let cell_x = x * 2;

        for dist in 0..trail_len {
            let draw_y = head_y - dist as i32;
            if draw_y >= 0 && draw_y < height as i32 && cell_x < width {
                let y = draw_y as usize;
                let color = trail_color(config, dist, trail_len);
                let ch = if dist == 0 {
                    col.head_char
                } else if dist < col.glyphs.len() {
                    col.glyphs[dist]
                } else {
                    continue;
                };
                rows[y].push((cell_x, ch, color));
            }
        }
    }

    for row in rows.iter_mut() {
        row.sort_by_key(|c| c.0);
    }

    rows
}

pub fn render_cells(
    stdout: &mut io::Stdout,
    rows: &[Vec<(usize, char, Color)>],
    area_width: u16,
    offset_x: u16,
    offset_y: u16,
    prev_occupied: &mut Vec<bool>,
) -> io::Result<()> {
    if prev_occupied.len() < rows.len() {
        prev_occupied.resize(rows.len(), false);
    }

    for (y, row) in rows.iter().enumerate() {
        let was_occupied = prev_occupied[y];
        let is_occupied = !row.is_empty();

        if !was_occupied && !is_occupied {
            continue;
        }

        queue!(stdout, cursor::MoveTo(offset_x, offset_y + y as u16))?;

        if is_occupied {
            let fill = " ".repeat(area_width as usize);
            queue!(stdout, Print(&fill))?;

            let mut cursor_x: Option<u16> = None;
            for &(x, ref ch, color) in row {
                let draw_x = offset_x + x as u16;
                if cursor_x != Some(draw_x) {
                    queue!(stdout, cursor::MoveTo(draw_x, offset_y + y as u16))?;
                }
                queue!(stdout, SetForegroundColor(color.into()), Print(ch))?;
                cursor_x = Some(draw_x + 1);
            }
        } else {
            let fill = " ".repeat(area_width as usize);
            queue!(stdout, Print(&fill))?;
        }

        prev_occupied[y] = is_occupied;
    }
    Ok(())
}

pub fn run_rain(config: &Config) -> io::Result<()> {
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    execute!(
        stdout,
        terminal::EnterAlternateScreen,
        cursor::Hide,
        terminal::Clear(ClearType::All)
    )?;

    let (mut width, mut height) = terminal::size()?;
    let mut num_columns = (width / 2) as usize;
    let density = config.density;
    let mut active_count = (num_columns as f32 * density) as usize;

    let mut columns: Vec<RainColumn> = (0..num_columns)
        .map(|i| {
            let mut col = RainColumn::random(config, height as f32);
            col.active = i < active_count;
            col
        })
        .collect();

    let frame_duration = Duration::from_millis(1000 / config.fps as u64);
    let mut prev_occupied: Vec<bool> = Vec::new();

    loop {
        let frame_start = Instant::now();

        if event::poll(Duration::from_millis(0))?
            && let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()?
        {
            match code {
                KeyCode::Char('q') | KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                    break;
                }
                _ => {}
            }
        }

        let (new_width, new_height) = terminal::size()?;
        if new_width != width || new_height != height {
            width = new_width;
            height = new_height;
            num_columns = (width / 2) as usize;
            active_count = (num_columns as f32 * density) as usize;
            columns.resize_with(num_columns, || RainColumn::random(config, height as f32));
            for (i, col) in columns.iter_mut().enumerate() {
                col.active = i < active_count;
            }
            execute!(stdout, terminal::Clear(ClearType::All))?;
            prev_occupied.clear();
        }

        for col in columns.iter_mut() {
            if !col.active {
                continue;
            }
            col.advance(height, config);
            col.glitch(config.glitch_frequency);
        }

        let rows = compute_cells(&columns, height as usize, width as usize, config);
        render_cells(&mut stdout, &rows, width, 0, 0, &mut prev_occupied)?;
        stdout.flush()?;

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }

    execute!(
        stdout,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;
    terminal::disable_raw_mode()?;
    Ok(())
}
