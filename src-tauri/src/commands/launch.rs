use std::sync::Arc;

use tauri::{AppHandle, Emitter};

use crate::commands::audio;
use crate::commands::gecko::install_gecko;
use crate::commands::process::drain_game_output;
use crate::commands::runners::resolve_effective_runner;
use crate::models::server::ServerConfig;
use crate::utils::{
    apply_game_env, effective_prefix, is_prefix_configured, pipe_output, wine_command,
    work_dir_from_exe, ExitEvent, EVENT_GAME_EXIT,
};
use crate::GameState;

#[tauri::command]
pub async fn launch_game(
    app: AppHandle,
    state: tauri::State<'_, GameState>,
    server: ServerConfig,
) -> Result<(), String> {
    let prefix = effective_prefix(server.wine_prefix.clone());

    if !is_prefix_configured(&prefix) {
        return Err(
            "El WINEPREFIX no está configurado. Ejecuta el setup primero.".to_string(),
        );
    }

    let resolved = resolve_effective_runner(server.runner.clone()).await?;

    install_gecko(&app, &prefix, &resolved.wine_bin).await?;
    audio::ensure_audio_driver(Some(&app), &prefix, &resolved).await?;

    let exe_path = server.executable_path.clone();
    let work_dir = work_dir_from_exe(&exe_path);

    let mut cmd = wine_command(
        &resolved.wine_bin,
        resolved.ld_library_path.as_deref(),
        &exe_path,
        &prefix,
        &work_dir,
        apply_game_env,
    );
    pipe_output(&mut cmd);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Error al lanzar el juego: {e}"))?;

    if let Some(pid) = child.id() {
        *state.pid.lock().unwrap() = Some(pid);
    }

    let pid_state = Arc::clone(&state.pid);
    tokio::spawn(async move {
        drain_game_output(&app, &mut child).await;
        let code = child
            .wait()
            .await
            .map(|s| s.code().unwrap_or(-1))
            .unwrap_or(-1);
        *pid_state.lock().unwrap() = None;
        let _ = app.emit(EVENT_GAME_EXIT, ExitEvent { code });
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_game(state: tauri::State<'_, GameState>) -> Result<(), String> {
    let pid = state.pid.lock().unwrap().take();
    if let Some(pid) = pid {
        let _ = tokio::process::Command::new("kill")
            .args(["-TERM", &pid.to_string()])
            .status()
            .await;
    }
    Ok(())
}
