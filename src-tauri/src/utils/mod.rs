use serde::{de::DeserializeOwned, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Emitter};
use tokio::io::AsyncBufReadExt;

pub fn app_data_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(format!("{}/.local/share/ro-launcher", home))
}

pub fn should_log_line(line: &str) -> bool {
    !line.contains("fixme:") && !line.contains("libEGL warning")
}

#[derive(Clone, Serialize)]
pub struct LogEvent {
    pub line: String,
}

pub async fn drain_and_log(app: &AppHandle, child: &mut tokio::process::Child) {
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();

    let app1 = app.clone();
    let h1 = tokio::spawn(async move {
        if let Some(out) = stdout {
            let mut lines = tokio::io::BufReader::new(out).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if should_log_line(&line) {
                    let _ = app1.emit("ro-launcher://log", LogEvent { line });
                }
            }
        }
    });

    let app2 = app.clone();
    let h2 = tokio::spawn(async move {
        if let Some(err) = stderr {
            let mut lines = tokio::io::BufReader::new(err).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if should_log_line(&line) {
                    let _ = app2.emit("ro-launcher://log", LogEvent { line });
                }
            }
        }
    });

    let _ = tokio::join!(h1, h2);
}

pub fn read_json<T: DeserializeOwned + Default>(path: &std::path::Path) -> Result<T, String> {
    if !path.exists() {
        return Ok(T::default());
    }
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

pub fn write_json<T: Serialize>(path: &std::path::Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    std::fs::write(path, json).map_err(|e| e.to_string())
}

pub fn work_dir_from_exe(exe_path: &str) -> String {
    std::path::Path::new(exe_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// Injects Proton LD_LIBRARY_PATH into a command if the runner needs it.
pub fn apply_runner_env(cmd: &mut tokio::process::Command, ld_library_path: Option<&str>) {
    if let Some(proton_libs) = ld_library_path {
        let lib_path = match std::env::var("LD_LIBRARY_PATH") {
            Ok(existing) if !existing.is_empty() => format!("{proton_libs}:{existing}"),
            _ => proton_libs.to_string(),
        };
        cmd.env("LD_LIBRARY_PATH", lib_path);
    }
}
