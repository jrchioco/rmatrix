use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

const DEFAULT_SPEED: f32 = 1.0;
const DEFAULT_DENSITY: f32 = 1.0;
const DEFAULT_FPS: u32 = 30;
const DEFAULT_BOLD: bool = false;
const DEFAULT_TRAIL_VARIABILITY: f32 = 0.5;
const DEFAULT_GLITCH_FREQUENCY: f32 = 0.1;
const DEFAULT_TRAIL_GLITCH_FREQUENCY: f32 = 0.05;
const DEFAULT_TRAIL_R: u8 = 0;
const DEFAULT_TRAIL_G: u8 = 255;
const DEFAULT_TRAIL_B: u8 = 65;
const DEFAULT_HEAD_R: u8 = 255;
const DEFAULT_HEAD_G: u8 = 255;
const DEFAULT_HEAD_B: u8 = 255;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub speed: f32,
    pub density: f32,
    pub fps: u32,
    pub bold: bool,
    pub trail_variability: f32,
    pub glitch_frequency: f32,
    pub trail_glitch_frequency: f32,
    pub trail_color: RgbColor,
    pub head_color: RgbColor,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            speed: DEFAULT_SPEED,
            density: DEFAULT_DENSITY,
            fps: DEFAULT_FPS,
            bold: DEFAULT_BOLD,
            trail_variability: DEFAULT_TRAIL_VARIABILITY,
            glitch_frequency: DEFAULT_GLITCH_FREQUENCY,
            trail_glitch_frequency: DEFAULT_TRAIL_GLITCH_FREQUENCY,
            trail_color: RgbColor {
                r: DEFAULT_TRAIL_R,
                g: DEFAULT_TRAIL_G,
                b: DEFAULT_TRAIL_B,
            },
            head_color: RgbColor {
                r: DEFAULT_HEAD_R,
                g: DEFAULT_HEAD_G,
                b: DEFAULT_HEAD_B,
            },
        }
    }
}

impl Config {
    pub fn config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home)
            .join(".config")
            .join("rmatrix")
            .join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        match fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str::<Config>(&contents) {
                Ok(config) => config.clamped(),
                Err(_) => Self::default(),
            },
            Err(_) => Self::default(),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("Failed to create config dir: {e}"))?;
        }
        let contents =
            toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize config: {e}"))?;
        fs::write(&path, contents).map_err(|e| format!("Failed to write config: {e}"))
    }

    pub fn clamped(mut self) -> Self {
        self.speed = self.speed.clamp(0.1, 5.0);
        self.density = self.density.clamp(0.0, 1.0);
        self.fps = self.fps.clamp(1, 60);
        self.trail_variability = self.trail_variability.clamp(0.0, 1.0);
        self.glitch_frequency = self.glitch_frequency.clamp(0.0, 1.0);
        self.trail_glitch_frequency = self.trail_glitch_frequency.clamp(0.0, 1.0);
        self
    }
}
