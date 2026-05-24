mod events;
mod json;
mod paths;
mod wine;

pub use events::*;
pub use json::*;
pub use paths::*;
pub use wine::*;

use tauri::{AppHandle, Emitter};
use tokio::io::AsyncBufReadExt;

pub fn should_log_line(line: &str) -> bool {
    !line.contains("fixme:") && !line.contains("libEGL warning")
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
                if should_log_line(&line) {
                    let _ = app2.emit(EVENT_LOG, LogEvent { line });
                }
            }
        }
    });

    let _ = tokio::join!(h1, h2);
}
