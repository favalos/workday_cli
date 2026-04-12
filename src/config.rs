use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub host: String,
    pub tenant: String,
    pub client_id: String,
    pub token_url: String,
    pub environment: String,
}

pub fn config_path() -> Result<PathBuf, String> {
    let home =
        std::env::var("HOME").map_err(|_| "HOME environment variable not set".to_string())?;
    Ok(PathBuf::from(home).join(".w-cli"))
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let dir = config_path()?;
    fs::create_dir_all(&dir).map_err(|e| format!("Failed to create config directory: {e}"))?;

    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;

    fs::write(dir.join("config.json"), &json)
        .map_err(|e| format!("Failed to write config file: {e}"))?;

    Ok(())
}

pub fn load_config() -> Result<Config, String> {
    let path = config_path()?.join("config.json");

    let content =
        fs::read_to_string(&path).map_err(|_| "Config not found. Run 'init' first.".to_string())?;

    serde_json::from_str::<Config>(&content).map_err(|e| format!("Failed to parse config: {e}"))
}
