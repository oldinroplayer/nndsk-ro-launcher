use std::path::Path;

use crate::commands::audio::{self, AudioDriver};
use crate::commands::runners::resolve_effective_runner;
use crate::models::dependency::DependencyStatus;
use crate::utils::{is_dxvk_installed, is_prefix_configured, prefix_path};

#[tauri::command]
pub async fn check_dependencies(runner: Option<String>) -> Result<DependencyStatus, String> {
    let wine = Path::new("/usr/bin/wine-cachyos").exists() || Path::new("/usr/bin/wine").exists();
    let winetricks = Path::new("/usr/bin/winetricks").exists();

    let prefix = prefix_path();
    let dxvk = is_dxvk_installed(&prefix);
    let prefix_configured = is_prefix_configured(&prefix);

    let resolved = resolve_effective_runner(runner).await?;

    let current_driver = if prefix_configured {
        audio::read_current_driver(&prefix, &resolved).await
    } else {
        None
    };

    let audio_status = audio::detect_audio_backends(current_driver);
    let audio_driver = audio_status
        .current_driver
        .or(Some(audio_status.recommended))
        .unwrap_or(AudioDriver::None);

    Ok(DependencyStatus {
        wine,
        winetricks,
        dxvk,
        prefix_configured,
        audio_ok: audio_status.ok,
        audio_driver: audio_driver.as_str().to_string(),
        audio_warning: audio_status.warning,
    })
}
