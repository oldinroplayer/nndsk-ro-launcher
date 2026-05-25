use tauri::AppHandle;
use tokio::process::Command;

use crate::utils::audio::{self, mmdevapi_recovery_hint};
use crate::utils::{drain_and_log, drain_child_output, pipe_output, should_log_line};

pub async fn run_logged_command(
    app: &AppHandle,
    mut cmd: Command,
    error_context: &str,
) -> Result<i32, String> {
    pipe_output(&mut cmd);

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("Error al ejecutar {error_context}: {e}"))?;

    drain_and_log(app, &mut child).await;

    let status = child.wait().await.map_err(|e| e.to_string())?;
    Ok(status.code().unwrap_or(-1))
}

pub async fn run_logged_command_ok(
    app: &AppHandle,
    cmd: Command,
    error_context: &str,
) -> Result<(), String> {
    let code = run_logged_command(app, cmd, error_context).await?;
    if code != 0 {
        return Err(format!("{error_context} falló con código: {code}"));
    }
    Ok(())
}

fn game_stderr_lines(line: &str) -> Vec<String> {
    let mut out = Vec::new();

    if audio::is_mmdevapi_audio_error(line) {
        out.push(mmdevapi_recovery_hint().to_string());
    }
    if line.contains("err:") || (should_log_line(line) && !line.is_empty()) {
        out.push(line.to_string());
    }

    out
}

/// stdout: filtra fixme. stderr: errores Wine + hints de audio mmdevapi.
pub async fn drain_game_output(app: &AppHandle, child: &mut tokio::process::Child) {
    drain_child_output(app, child, game_stderr_lines).await;
}
