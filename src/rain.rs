use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, ClearType},
};
use rand::Rng;

use crate::config::Config;

const BAYBAYIN: [char; 23] = [
    '\u{1700}', '\u{1701}', '\u{1702}', '\u{1703}', '\u{1704}', '\u{1705}',
    '\u{1706}', '\u{1707}', '\u{1708}', '\u{1709}', '\u{170A}', '\u{170B}',
    '\u{170C}', '\u{170D}', '\u{170E}', '\u{170F}', '\u{1710}', '\u{1711}',
    '\u{1712}', '\u{1713}', '\u{1714}', '\u{1715}', '\u{1716}',
];

struct Column {
    y: f32,
    speed: f32,
    length: u16,
    active: bool,
    glyphs: Vec<char>,
    head_char: char,
}

impl Column {
    fn new(y: f32, speed: f32, length: u16) -> Self {
        let mut rng = rand::thread_rng();
        let glyphs: Vec<char> = (0..length)
            .map(|_| BAYBAYIN[rng.gen_range(0..BAYBAYIN.len())])
            .collect();
        let head_char = BAYBAYIN[rng.gen_range(0..BAYBAYIN.len())];
        Self {
            y,
            speed,
            length,
            active: true,
            glyphs,
            head_char,
        }
    }

    fn reset(&mut self, config: &Config) {
        let mut rng = rand::thread_rng();
        let base_length = 8.0_f32;
        let variation = config.trail_variability * 12.0;
        self.length = (base_length + rng.gen_range(0.0..1.0) * variation) as u16;
        self.length = self.length.max(3);
        self.speed = config.speed * (0.5 + rng.gen_range(0.0..1.0) * 1.0);
        self.y = -(rng.gen_range(0.0..1.0) * 5.0);
        self.glyphs = (0..self.length)
            .map(|_| BAYBAYIN[rng.gen_range(0..BAYBAYIN.len())])
            .collect();
        self.head_char = BAYBAYIN[rng.gen_range(0..BAYBAYIN.len())];
        self.active = true;
    }

    fn advance(&mut self, height: u16, config: &Config) {
        self.y += self.speed;
        if self.y - self.length as f32 > height as f32 {
            self.reset(config);
        }
    }

    fn glitch(&mut self, frequency: f32) {
        let mut rng = rand::thread_rng();
        if rng.gen_range(0.0..1.0) < frequency {
            let idx = rng.gen_range(0..self.glyphs.len());
            self.glyphs[idx] = BAYBAYIN[rng.gen_range(0..BAYBAYIN.len())];
        }
        if rng.gen_range(0.0..1.0) < frequency {
            self.head_char = BAYBAYIN[rng.gen_range(0..BAYBAYIN.len())];
        }
    }
}

fn trail_color(config: &Config, distance_from_head: usize, trail_length: usize) -> Color {
    if distance_from_head == 0 {
        return Color::Rgb {
            r: config.head_color.r,
            g: config.head_color.g,
            b: config.head_color.b,
        };
    }
    let ratio = 1.0 - (distance_from_head as f32 / trail_length as f32);
    let base_r = config.trail_color.r as f32;
    let base_g = config.trail_color.g as f32;
    let base_b = config.trail_color.b as f32;
    let intensity = ratio * 0.7 + 0.1;
    Color::Rgb {
        r: (base_r * intensity) as u8,
        g: (base_g * intensity) as u8,
        b: (base_b * intensity) as u8,
    }
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
    let mut num_columns = width as usize;
    let density = config.density;
    let mut active_count = (num_columns as f32 * density) as usize;

    let mut rng = rand::thread_rng();
    let mut columns: Vec<Column> = (0..num_columns)
        .map(|i| {
            let is_active = i < active_count;
            let speed = config.speed * (0.5 + rng.gen_range(0.0..1.0) * 1.0);
            let base_length = 8.0_f32;
            let variation = config.trail_variability * 12.0;
            let length = (base_length + rng.gen_range(0.0..1.0) * variation) as u16;
            let y = rng.gen_range(0.0..1.0) * height as f32;
            let mut col = Column::new(y, speed, length.max(3));
            col.active = is_active;
            col
        })
        .collect();

    let frame_duration = Duration::from_millis(1000 / config.fps as u64);

    loop {
        let frame_start = Instant::now();

        if event::poll(Duration::from_millis(0))? {
            if let Event::Key(KeyEvent { code, modifiers, .. }) = event::read()? {
                match code {
                    KeyCode::Char('q') | KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        let (new_width, new_height) = terminal::size()?;
        if new_width != width || new_height != height {
            width = new_width;
            height = new_height;
            num_columns = width as usize;
            active_count = (num_columns as f32 * density) as usize;
            columns.resize_with(num_columns, || {
                let speed = config.speed * (0.5 + rng.gen_range(0.0..1.0) * 1.0);
                let base_length = 8.0_f32;
                let variation = config.trail_variability * 12.0;
                let length = (base_length + rng.gen_range(0.0..1.0) * variation) as u16;
                let y = rng.gen_range(0.0..1.0) * height as f32;
                Column::new(y, speed, length.max(3))
            });
            for (i, col) in columns.iter_mut().enumerate() {
                col.active = i < active_count;
            }
            execute!(stdout, terminal::Clear(ClearType::All))?;
        }

        for col in columns.iter_mut() {
            if !col.active {
                continue;
            }
            col.advance(height, config);
            col.glitch(config.glitch_frequency);
        }

        execute!(stdout, cursor::MoveTo(0, 0))?;
        let mut screen = vec![vec![' '; width as usize]; height as usize];
        let mut colors = vec![vec![Color::Black; width as usize]; height as usize];

        for (x, col) in columns.iter().enumerate() {
            if !col.active {
                continue;
            }
            let head_y = col.y as i32;
            let trail_len = col.length as usize;

            for dist in 0..trail_len {
                let draw_y = head_y - dist as i32;
                if draw_y >= 0 && draw_y < height as i32 {
                    let y = draw_y as usize;
                    let color = trail_color(config, dist, trail_len);
                    let ch = if dist == 0 {
                        col.head_char
                    } else if dist < col.glyphs.len() {
                        col.glyphs[dist]
                    } else {
                        ' '
                    };
                    screen[y][x] = ch;
                    colors[y][x] = color;
                }
            }
        }

        for (y, row) in screen.iter().enumerate() {
            execute!(stdout, cursor::MoveTo(0, y as u16))?;
            for (x, &ch) in row.iter().enumerate() {
                execute!(stdout, SetForegroundColor(colors[y][x]), Print(ch))?;
            }
        }
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
