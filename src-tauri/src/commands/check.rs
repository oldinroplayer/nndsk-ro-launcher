use crate::utils::audio;
use crate::models::dependency::DependencyStatus;
use crate::utils::resolve_effective_runner;
use crate::utils::{
    is_dxvk_installed, is_prefix_configured, prefix_path, system_wine_available,
    winetricks_available,
};

#[tauri::command]
pub async fn check_dependencies(runner: Option<String>) -> Result<DependencyStatus, String> {
    let wine = system_wine_available();
    let winetricks = winetricks_available();

    let prefix = prefix_path();
    let dxvk = is_dxvk_installed(&prefix);
    let prefix_configured = is_prefix_configured(&prefix);

    let resolved = resolve_effective_runner(runner).await?;
    let (audio_ok, audio_driver, audio_warning) =
        audio::dependency_audio_fields(&prefix, prefix_configured, &resolved).await;

    Ok(DependencyStatus {
        wine,
        winetricks,
        dxvk,
        prefix_configured,
        audio_ok,
        audio_driver,
        audio_warning,
    })
}
