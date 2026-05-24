use serde::Serialize;
use tauri::{AppHandle, Emitter};

pub const EVENT_LOG: &str = "ro-launcher://log";
pub const EVENT_PROGRESS: &str = "ro-launcher://progress";
pub const EVENT_GAME_EXIT: &str = "ro-launcher://game-exit";
pub const EVENT_ERROR: &str = "ro-launcher://error";

#[derive(Clone, Serialize)]
pub struct LogEvent {
    pub line: String,
}

#[derive(Clone, Serialize)]
pub struct ProgressEvent {
    pub step: String,
    pub percent: u32,
}

#[derive(Clone, Serialize)]
pub struct ExitEvent {
    pub code: i32,
}

#[derive(Clone, Serialize)]
pub struct ErrorEvent {
    pub message: String,
}

pub fn emit_log(app: &AppHandle, line: impl Into<String>) -> Result<(), String> {
    app.emit(EVENT_LOG, LogEvent { line: line.into() })
        .map_err(|e| e.to_string())
}

pub fn emit_log_opt(app: Option<&AppHandle>, line: impl Into<String>) {
    if let Some(app) = app {
        let _ = app.emit(EVENT_LOG, LogEvent { line: line.into() });
    }
}

pub fn emit_progress(app: &AppHandle, step: &str, percent: u32) -> Result<(), String> {
    app.emit(
        EVENT_PROGRESS,
        ProgressEvent {
            step: step.to_string(),
            percent,
        },
    )
    .map_err(|e| e.to_string())
}

/// Reservado para errores asíncronos fuera del flujo invoke (p. ej. tareas en background).
#[allow(dead_code)]
pub fn emit_error(app: &AppHandle, message: impl Into<String>) -> Result<(), String> {
    app.emit(
        EVENT_ERROR,
        ErrorEvent {
            message: message.into(),
        },
    )
    .map_err(|e| e.to_string())
}
