use serde::{Deserialize, Serialize};
use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::PathBuf;

fn default_r() -> u8 { 137 }
fn default_g() -> u8 { 180 }
fn default_b() -> u8 { 250 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub active_profile: u8,
    pub active_dpi_stage: u8,
    pub dpi_stages: [u16; 6],
    pub polling_rate_hz: i32,
    pub lod_code: u8,
    pub debounce_ms: u8,
    pub motion_sync: bool,
    pub ripple_control: bool,
    pub sleep_timeout_mins: u8,
    pub lighting_mode_code: u8,
    pub lighting_brightness: u8,
    pub lighting_speed: u8,
    #[serde(default = "default_r")]
    pub lighting_color_r: u8,
    #[serde(default = "default_g")]
    pub lighting_color_g: u8,
    #[serde(default = "default_b")]
    pub lighting_color_b: u8,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            active_profile: 1,
            active_dpi_stage: 1,
            dpi_stages: [800, 1600, 2400, 3200, 5000, 12000],
            polling_rate_hz: 1000,
            lod_code: 0,
            debounce_ms: 4,
            motion_sync: true,
            ripple_control: false,
            sleep_timeout_mins: 5,
            lighting_mode_code: 3,
            lighting_brightness: 80,
            lighting_speed: 5,
            lighting_color_r: 137,
            lighting_color_g: 180,
            lighting_color_b: 250,
        }
    }
}

fn config_path() -> PathBuf {
    let mut dir = dirs_config();
    dir.push("glitch-r5u");
    let _ = create_dir_all(&dir);
    dir.push("config.json");
    dir
}

fn dirs_config() -> PathBuf {
    if let Ok(path) = std::env::var("XDG_CONFIG_HOME") {
        PathBuf::from(path)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".config")
    } else {
        PathBuf::from("/tmp")
    }
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    if let Ok(mut f) = File::open(&path) {
        let mut content = String::new();
        if f.read_to_string(&mut content).is_ok() {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                return cfg;
            }
        }
    }
    // Legacy migration fallback
    let legacy_path = dirs_config().join("attackshark-control").join("config.json");
    if let Ok(mut f) = File::open(&legacy_path) {
        let mut content = String::new();
        if f.read_to_string(&mut content).is_ok() {
            if let Ok(cfg) = serde_json::from_str::<AppConfig>(&content) {
                let _ = save_config(&cfg);
                return cfg;
            }
        }
    }
    AppConfig::default()
}

pub fn save_config(cfg: &AppConfig) -> std::io::Result<()> {
    let path = config_path();
    let json = serde_json::to_string_pretty(cfg)?;
    let mut f = File::create(&path)?;
    f.write_all(json.as_bytes())?;
    Ok(())
}
