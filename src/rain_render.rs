use ratatui::style::Color;
use rand::Rng;

use crate::config::Config;

pub const BAYBAYIN: [char; 23] = [
    '\u{1700}', '\u{1701}', '\u{1702}', '\u{1703}', '\u{1704}', '\u{1705}',
    '\u{1706}', '\u{1707}', '\u{1708}', '\u{1709}', '\u{170A}', '\u{170B}',
    '\u{170C}', '\u{170D}', '\u{170E}', '\u{170F}', '\u{1710}', '\u{1711}',
    '\u{1712}', '\u{1713}', '\u{1714}', '\u{1715}', '\u{1716}',
];

pub struct RainColumn {
    pub y: f32,
    pub speed: f32,
    pub length: u16,
    pub active: bool,
    pub glyphs: Vec<char>,
    pub head_char: char,
}

impl RainColumn {
    pub fn new(y: f32, speed: f32, length: u16) -> Self {
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

    pub fn reset(&mut self, config: &Config, _height: f32) {
        let mut rng = rand::thread_rng();
        let base_length = 8.0_f32;
        let variation = config.trail_variability * 42.0;
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

    pub fn advance(&mut self, height: u16, config: &Config) {
        self.y += self.speed;
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
        let head_y = col.y as i32;
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
