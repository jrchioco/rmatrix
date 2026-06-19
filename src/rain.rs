use std::io::{self, Write};
use std::time::{Duration, Instant};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Print, SetForegroundColor},
    terminal::{self, ClearType},
};
use rand::Rng;

use crate::config::Config;
use crate::rain_render::{compute_cells, RainColumn};

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

    let mut rng = rand::thread_rng();
    let mut columns: Vec<RainColumn> = (0..num_columns)
        .map(|i| {
            let is_active = i < active_count;
            let speed = config.speed * (0.5 + rng.gen_range(0.0..1.0) * 1.0);
            let base_length = 8.0_f32;
            let variation = config.trail_variability * 42.0;
            let length = (base_length + rng.gen_range(0.0..1.0) * variation) as u16;
            let y = rng.gen_range(0.0..1.0) * height as f32;
            let mut col = RainColumn::new(y, speed, length.max(3));
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
            num_columns = (width / 2) as usize;
            active_count = (num_columns as f32 * density) as usize;
            columns.resize_with(num_columns, || {
                let speed = config.speed * (0.5 + rng.gen_range(0.0..1.0) * 1.0);
                let base_length = 8.0_f32;
                let variation = config.trail_variability * 12.0;
                let length = (base_length + rng.gen_range(0.0..1.0) * variation) as u16;
                let y = rng.gen_range(0.0..1.0) * height as f32;
                RainColumn::new(y, speed, length.max(3))
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

        let rows = compute_cells(&columns, height as usize, width as usize, config);

        for (y, row) in rows.iter().enumerate() {
            execute!(stdout, cursor::MoveTo(0, y as u16))?;
            let fill = " ".repeat(width as usize);
            execute!(stdout, Print(&fill))?;

            let mut cursor_x = 0u16;
            for &(x, ref ch, color) in row {
                if x as u16 > cursor_x {
                    cursor_x = x as u16;
                    execute!(stdout, cursor::MoveTo(cursor_x, y as u16))?;
                }
                execute!(stdout, SetForegroundColor(color.into()), Print(ch))?;
                cursor_x += 1;
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
