# baybayin-rain — Build Plan

## Goal
Terminal "Matrix digital rain" effect using Baybayin script instead of katakana/alphabet. Rust binary, single crate, no AUR/system deps beyond cargo.

## Stack
- Rust (stable, edition 2021)
- `crossterm = "0.27"` — terminal control (raw mode, cursor, color)
- `rand = "0.8"` — glyph/speed randomization

## Project Setup
```
cargo init baybayin-rain
cd baybayin-rain
cargo add crossterm rand
```

## Charset
Baybayin Unicode block: U+1700–U+171A (skip U+171B–U+171F, reserved/punctuation, often unsupported by fonts).
Valid codepoints to sample from:
```rust
const BAYBAYIN: [char; 23] = [
    '\u{1700}','\u{1701}','\u{1702}','\u{1703}','\u{1704}','\u{1705}',
    '\u{1706}','\u{1707}','\u{1708}','\u{1709}','\u{170A}','\u{170B}',
    '\u{170C}','\u{170D}','\u{170E}','\u{170F}','\u{1710}','\u{1711}',
    '\u{1712}','\u{1713}','\u{1714}','\u{1715}', '\u{1716}',
];
```
Agent should verify each codepoint renders (no tofu boxes) in target terminal/font before finalizing array — Noto Sans Tagalog or similar font required on the system.

## Architecture
1. **Terminal setup**: enable raw mode, hide cursor, enter alternate screen via crossterm.
2. **Column state**: `Vec<Column>` sized to terminal width. Each `Column` has:
   - `y: f32` — current head position
   - `speed: f32` — randomized per column
   - `length: u16` — trail length
3. **Render loop** (target ~20-30 fps, sleep-based, no async needed):
   - Clear or partially fade previous frame (print trailing dim/black chars above head to simulate fade, like cmatrix does — don't full-clear every frame, it flickers)
   - For each column: print bright glyph at head (white/light-green), dimmer green glyphs in trail
   - Random glyph swap each frame for "digital" flicker feel
   - Advance head position by speed; reset to top with new random speed when off-screen
4. **Color**: use crossterm `Color::Rgb` — head bright white/pale green, trail fading dark green via several shades.
5. **Exit handling**: listen for `q` or `Ctrl+C` via crossterm event poll (non-blocking, short timeout per frame), restore terminal (disable raw mode, leave alternate screen, show cursor) on exit — **must run on panic too**, use a guard/drop impl or catch panics to avoid leaving terminal broken.
6. **Resize handling**: optional v2 — recompute column count on terminal resize event.

## Guardrails for the agent
- Do NOT skip terminal-state restoration on exit/panic — broken terminal (no cursor, raw mode stuck) is the #1 failure mode for this kind of project.
- Keep frame loop simple: `std::thread::sleep`, no async runtime needed.
- Test incrementally: (1) static glyph print test → (2) single falling column → (3) full multi-column with fade → (4) color/speed tuning.
- No external network calls, no unsafe blocks needed for this project — flag if agent adds either.

## CLI Controls (v1, via `clap`)

```
baybayin-rain [OPTIONS]

--speed <0.1-5.0>      Fall speed multiplier (default 1.0)
--density <0.0-1.0>    Fraction of columns active at once (default 0.6)
--color <NAME|HEX>     Trail color (default "green")
--head-color <NAME|HEX> Bright leading glyph color (default "white")
--fps <1-60>           Render rate (default 30)
--trail-length <N>     Glyph trail length per column (default 8-20 randomized if unset)
```

### Color input
Accept both named presets and hex:
- Named: `green`, `amber`, `cyan`, `red`, `white`, `purple` — map each to an RGB triple internally.
- Hex: `--color=#00FF41` or `00FF41` (with/without `#`), parsed as RGB. `F0055` (5 chars) is invalid — must be 6 hex digits (RRGGBB). Validate length/parse and reject with a clear error message ("expected 6 hex digits, e.g. 00FF41"), don't silently fall back.

```rust
fn parse_color(s: &str) -> Result<Color, String> {
    let s = s.trim_start_matches('#');
    if let Some(named) = match_named_color(s) { return Ok(named); }
    if s.len() != 6 || !s.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!("invalid color '{s}': use a name (green/amber/cyan/...) or 6-digit hex (e.g. 00FF41)"));
    }
    let r = u8::from_str_radix(&s[0..2], 16).unwrap();
    let g = u8::from_str_radix(&s[2..4], 16).unwrap();
    let b = u8::from_str_radix(&s[4..6], 16).unwrap();
    Ok(Color::Rgb { r, g, b })
}
```

### Validation rules
- `speed`: clamp to [0.1, 5.0], warn if out of range rather than crash.
- `density`: clamp to [0.0, 1.0].
- `fps`: clamp to [1, 60] (above 60 wastes CPU with no visible benefit).
- Reject, don't silently clamp, malformed colors — bad color = typo, bad number = user pushing limits intentionally.

## Stretch (v2, only after v1 works)
- Config file for color themes (so flags aren't retyped every run)
- Bold/highlight random glyph occasionally (cmatrix "glitch" effect)
- `--theme=matrix|amber-crt|cyberpunk` presets bundling color+speed+density

## Definition of Done
- `cargo run` shows falling Baybayin rain, green-on-black, smooth (~30fps), clean `q` exit, terminal state fully restored after exit.
