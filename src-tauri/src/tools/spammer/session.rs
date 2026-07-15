use std::sync::Arc;

use ro_tools_core::{CombatInputBackend, SpammerConfig};
use tauri::AppHandle;

use crate::tools::input::{ensure_ydotoold, InputGateway, YdotoolDaemon};
use crate::utils::emit_tool_log_opt;

use super::SpammerHandle;

/// Arranca el loop de spam (solo input virtual; no requiere PID).
pub async fn start_session(
    app: AppHandle,
    handle: &SpammerHandle,
    input: InputGateway,
    ydotoold: Arc<YdotoolDaemon>,
    config: SpammerConfig,
    backend: CombatInputBackend,
) -> Result<(), String> {
    match backend {
        CombatInputBackend::Ydotool => {
            emit_tool_log_opt(Some(&app), "[Spammer] Preparando ydotool...");
            ensure_ydotoold(Some(&app), ydotoold.as_ref()).await?;
        }
        CombatInputBackend::Uinput if !input.is_uinput_prepared() => {
            return Err("Spammer no puede iniciar: uinput no fue preparado antes de Wine".into());
        }
        CombatInputBackend::Uinput => {}
    }
    handle.start(app, input, config, backend, ydotoold).await
}
