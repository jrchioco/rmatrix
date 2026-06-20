# rmatrix

Matrix digital rain effect using **Baybayin script** (ᜀ–ᜑ), the indigenous writing system of the Philippines.

> **Note:** This project was vibe coded — built through rapid iteration and experimentation with AI assistance. It works, but the code may reflect that journey.

## Features

- Falling Baybayin glyphs with colored trails that fade from bright head to dim tail
- Interactive settings TUI with live preview (`--settings`)
- Persistent config saved to `~/.config/rmatrix/config.toml`
- Customizable speed, density, FPS, colors, trail variability, glitch frequency, and bold mode
- Double-width character support for proper Baybayin rendering
- Smooth performance via batched terminal writes

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
git clone <repo-url>
cd rmatrix
cargo build --release
```

## Usage

```bash
# Run rain with saved settings
rmatrix

# Open settings TUI, then start rain
rmatrix --settings
```

### Controls (rain)

| Key | Action |
|-----|--------|
| `q` / `Ctrl+C` | Quit |

### Controls (settings TUI)

| Key | Action |
|-----|--------|
| `↑`/`↓` or `Tab` | Select field |
| `←`/`→` | Adjust value |
| `Enter` | Toggle bold |
| `q` / `Esc` | Save & start rain |

## Configuration

Settings are stored at `~/.config/rmatrix/config.toml`:

```toml
speed = 1.0            # 0.1–5.0
density = 1.0          # 0.0–1.0
fps = 30               # 1–60
bold = false
trail_variability = 0.5 # 0.0–1.0
glitch_frequency = 0.1  # 0.0–1.0

[trail_color]
r = 0
g = 255
b = 65

[head_color]
r = 255
g = 255
b = 255
```

## Requirements

- A terminal font supporting Baybayin (U+1700–U+1711), e.g. [Noto Sans Tagalog](https://fonts.google.com/noto/specimen/Noto+Sans+Tagalog)
- Rust toolchain (edition 2024)

## Dependencies

| Crate | Purpose |
|-------|---------|
| crossterm | Terminal I/O, raw mode, alternate screen |
| ratatui | Settings TUI widgets & layout |
| clap | CLI argument parsing |
| rand | Randomization |
| serde + toml | Config serialization |

## License

MIT
