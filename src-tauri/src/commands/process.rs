use tauri::{AppHandle, Emitter};
use tokio::io::AsyncBufReadExt;
use tokio::process::Command;

use crate::commands::audio::{self, mmdevapi_recovery_hint};
use crate::utils::{drain_and_log, pipe_output, should_log_line, LogEvent, EVENT_LOG};

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

/// stdout: filtra fixme. stderr: errores Wine + hints de audio mmdevapi.
pub async fn drain_game_output(app: &AppHandle, child: &mut tokio::process::Child) {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let app1 = app.clone();
    let h1 = tokio::spawn(async move {
        if let Some(out) = stdout {
            let mut lines = tokio::io::BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if should_log_line(&line) {
                    let _ = app1.emit(EVENT_LOG, LogEvent { line });
                }
            }
        }
    });

    let app2 = app.clone();
    let h2 = tokio::spawn(async move {
        if let Some(err) = stderr {
            let mut lines = tokio::io::BufReader::new(err).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if audio::is_mmdevapi_audio_error(&line) {
                    let _ = app2.emit(EVENT_LOG, LogEvent {
                        line: mmdevapi_recovery_hint().to_string(),
                    });
                }
                if line.contains("err:") || (should_log_line(&line) && !line.is_empty()) {
                    let _ = app2.emit(EVENT_LOG, LogEvent { line });
                }
            }
        }
    });

    let _ = tokio::join!(h1, h2);
}
