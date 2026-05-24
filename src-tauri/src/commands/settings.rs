use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::utils::{app_data_dir, read_json, write_json};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub default_runner: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        let runner = if Path::new("/usr/bin/wine-cachyos").exists() {
            "/usr/bin/wine-cachyos"
        } else {
            "/usr/bin/wine"
        };
        Self {
            default_runner: runner.to_string(),
        }
    }
}

fn settings_path() -> std::path::PathBuf {
    app_data_dir().join("settings.json")
}

pub async fn effective_runner(override_path: Option<String>) -> Result<String, String> {
    match override_path {
        Some(path) if !path.is_empty() => Ok(path),
        _ => Ok(load_settings().await?.default_runner),
    }
}

#[tauri::command]
pub async fn load_settings() -> Result<AppSettings, String> {
    read_json(&settings_path())
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String> {
    write_json(&settings_path(), &settings)
}
