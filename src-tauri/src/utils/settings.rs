use std::path::PathBuf;

use crate::models::settings::AppSettings;
use crate::utils::{data_file, read_json, write_json};

pub fn settings_path() -> PathBuf {
    data_file("settings.json")
}

pub async fn load_app_settings() -> Result<AppSettings, String> {
    read_json(&settings_path())
}

pub async fn save_app_settings(settings: &AppSettings) -> Result<(), String> {
    write_json(&settings_path(), settings)
}

pub async fn effective_runner(override_path: Option<String>) -> Result<String, String> {
    match override_path {
        Some(path) if !path.is_empty() => Ok(path),
        _ => Ok(load_app_settings().await?.default_runner),
    }
}
