use crate::models::settings::AppSettings;
use crate::utils::{load_app_settings, save_app_settings};

#[tauri::command]
pub async fn load_settings() -> Result<AppSettings, String> {
    load_app_settings().await
}

#[tauri::command]
pub async fn save_settings(settings: AppSettings) -> Result<(), String> {
    save_app_settings(&settings).await
}
