mod gateway;
mod uinput_worker;
mod ydotool;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::utils::emit_tool_log_opt;

pub use gateway::{GatewayWriter, InputGateway};
pub use uinput_worker::InputSource;
pub use ydotool::{dependency_ydotool_input_fields, ensure_ydotoold, YdotoolDaemon};

pub(crate) fn emit_status_if_changed<T>(
    app: &AppHandle,
    status_arc: &Arc<Mutex<T>>,
    event_name: &str,
    event: T,
) where
    T: Clone + PartialEq + Serialize,
{
    let mut prev = status_arc.lock().unwrap();
    if *prev != event {
        *prev = event.clone();
        drop(prev);
        let _ = app.emit(event_name, event);
    }
}

pub(crate) async fn recover_ydotool_on_error(
    app: &AppHandle,
    gateway: &InputGateway,
    ydotoold: &YdotoolDaemon,
    last_recovery: &mut Instant,
    err_msg: &str,
    success_log: &str,
) -> bool {
    if !err_msg.contains("ydotool") || last_recovery.elapsed() < Duration::from_secs(5) {
        return false;
    }

    *last_recovery = Instant::now();
    if ensure_ydotoold(Some(app), ydotoold).await.is_ok() {
        gateway.reset_ydotool();
        emit_tool_log_opt(Some(app), success_log);
        return true;
    }
    false
}
