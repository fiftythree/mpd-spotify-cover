use std::fs;
use std::path::Path;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub spotify: SpotifyConfig,
    pub mpd: MpdConfig,
    pub cover: CoverConfig,
}

#[derive(Serialize, Deserialize)]
pub struct SpotifyConfig {
    pub client_id: String,
    pub client_secret: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
}

#[derive(Serialize, Deserialize)]
pub struct MpdConfig {
    pub address: String,
    pub port: u16,
}

#[derive(Serialize, Deserialize)]
pub struct CoverConfig {
    pub preferred_size: String,
    pub output_path: String,
}

impl Config {

    pub fn auth_configured(&self) -> bool {
        let spotify_config = &self.spotify;

        return spotify_config.access_token.is_some()
            && spotify_config.refresh_token.is_some()
            && spotify_config.expires_at.is_some();
    }
}

pub fn read_config() -> Result<Config, String> {
    let path = Path::new("config.toml");

    toml::from_slice(fs::read_to_string(path)
        .map_err(|err| format!(
            "Couldn't read the config file: {}",
            err.to_string())
        )?.as_bytes()
    ).map_err(|err| format!(
        "Couldn't deserialize config: {}",
        err.to_string()
    ))
}

pub fn save_config(config: &Config) -> Result<(), String> {
    let contents = toml::to_string(&config)
        .map_err(|_| "unable to serialize config")?;

    fs::write(Path::new("config.toml"), contents)
        .map_err(|e| e.to_string())
}
