use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ro_tools_core::{InputWriter, SpammerConfig, SpammerEngine};
use tauri::AppHandle;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::watch;
use tokio::time::{interval, MissedTickBehavior};

use crate::models::spammer::SpammerStatusEvent;
use crate::tools::input::{
    emit_status_if_changed as emit_status_event, recover_ydotool_on_error as recover_ydotool_event,
    InputGateway, YdotoolDaemon,
};
use crate::utils::{emit_tool_log_opt, EVENT_SPAMMER_STATUS};

const KEY_POLL_MS: u64 = 5;
const INPUTD_READY_TIMEOUT_SECS: u64 = 5;

#[derive(Debug)]
enum InputdMsg {
    Ready,
    TriggerHeld(bool),
    Fatal(String),
}

fn parse_line(line: &str) -> Option<InputdMsg> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    match v.get("type")?.as_str()? {
        "ready" => Some(InputdMsg::Ready),
        "trigger" => Some(InputdMsg::TriggerHeld(v.get("held")?.as_bool()?)),
        "fatal" => Some(InputdMsg::Fatal(
            v.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("fatal")
                .to_string(),
        )),
        _ => None,
    }
}

fn emit_status_if_changed(
    app: &AppHandle,
    status_arc: &Arc<Mutex<SpammerStatusEvent>>,
    event: SpammerStatusEvent,
) {
    emit_status_event(app, status_arc, EVENT_SPAMMER_STATUS, event);
}

async fn recover_ydotool_on_error(
    app: &AppHandle,
    gateway: &InputGateway,
    ydotoold: &YdotoolDaemon,
    last_recovery: &mut Instant,
    err_msg: &str,
) -> bool {
    recover_ydotool_event(
        app,
        gateway,
        ydotoold,
        last_recovery,
        err_msg,
        "[Input] ydotoold recuperado (spammer)",
    )
    .await
}

fn find_ro_inputd() -> std::path::PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let plain = dir.join("ro-inputd");
            if plain.exists() {
                return plain;
            }
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name();
                    let name = name.to_string_lossy();
                    if name == "ro-inputd" || name.starts_with("ro-inputd-") {
                        return entry.path();
                    }
                }
            }
        }
    }
    std::path::PathBuf::from("ro-inputd")
}

pub async fn run(
    app: AppHandle,
    writer: crate::tools::input::GatewayWriter,
    config: SpammerConfig,
    mut stop_rx: watch::Receiver<bool>,
    status_arc: Arc<Mutex<SpammerStatusEvent>>,
    gateway: InputGateway,
    ydotoold: Arc<YdotoolDaemon>,
) {
    let inputd_path = find_ro_inputd();

    let mut child = match tokio::process::Command::new(&inputd_path)
        .arg("--trigger")
        .arg("F1")
        .arg("--json")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let msg = format!("[Spammer] ro-inputd no encontrado ({inputd_path:?}): {e}");
            emit_tool_log_opt(Some(&app), &msg);
            emit_status_if_changed(
                &app,
                &status_arc,
                SpammerStatusEvent {
                    active: false,
                    error: Some(msg),
                    ..SpammerStatusEvent::default()
                },
            );
            return;
        }
    };

    let mut stdin = child.stdin.take().expect("stdin piped");
    let mut lines = BufReader::new(child.stdout.take().expect("stdout piped")).lines();

    emit_tool_log_opt(Some(&app), "[Spammer] Esperando ro-inputd...");

    let engine = Arc::new(Mutex::new(SpammerEngine::new(writer, config.clone())));
    let mut poll = spammer_poll_ticker();
    let mut last_spam = Instant::now()
        .checked_sub(Duration::from_secs(1))
        .unwrap_or_else(Instant::now);
    let mut last_log_cycle: u64 = 0;
    let mut cycle_count: u64 = 0;
    let mut spamming = false;
    let mut trigger_held = false;
    let mut ready_received = false;
    let mut last_ydotool_recovery = Instant::now()
        .checked_sub(Duration::from_secs(10))
        .unwrap_or_else(Instant::now);

    let ready_timeout = tokio::time::sleep(Duration::from_secs(INPUTD_READY_TIMEOUT_SECS));
    tokio::pin!(ready_timeout);

    'main: loop {
        tokio::select! {
            line_result = lines.next_line() => {
                match line_result {
                    Ok(Some(line)) => match parse_line(&line) {
                        Some(InputdMsg::Ready) => {
                            ready_received = true;
                            emit_tool_log_opt(Some(&app), "[Spammer] ro-inputd listo — grab activo");
                            let _ = gateway.writer().key_up("F1");
                            emit_status_if_changed(&app, &status_arc, SpammerStatusEvent {
                                active: true,
                                armed: true,
                                spamming: false,
                                key: "F1".to_string(),
                                delay_ms: config.delay_ms,
                                cycle_count: 0,
                                error: None,
                            });
                        }
                        Some(InputdMsg::TriggerHeld(held)) if ready_received => {
                            let was_held = trigger_held;
                            trigger_held = held;
                            if held && !was_held {
                                spamming = true;
                                last_spam = Instant::now()
                                    .checked_sub(Duration::from_millis(config.delay_ms))
                                    .unwrap_or_else(Instant::now);
                                emit_tool_log_opt(Some(&app), "[Spammer] Spam activo (F1)");
                            } else if !held {
                                spamming = false;
                            }
                        }
                        Some(InputdMsg::Fatal(msg)) => {
                            emit_tool_log_opt(Some(&app), format!("[Spammer] Fatal ro-inputd: {msg}"));
                            emit_status_if_changed(&app, &status_arc, SpammerStatusEvent {
                                active: false,
                                error: Some(msg),
                                ..SpammerStatusEvent::default()
                            });
                            break 'main;
                        }
                        _ => {}
                    }
                    _ => {
                        let msg = "[Spammer] ro-inputd terminó inesperadamente".to_string();
                        emit_tool_log_opt(Some(&app), &msg);
                        emit_status_if_changed(&app, &status_arc, SpammerStatusEvent {
                            active: false,
                            error: Some(msg),
                            ..SpammerStatusEvent::default()
                        });
                        break 'main;
                    }
                }
            }
            _ = poll.tick(), if ready_received => {
                if trigger_held && last_spam.elapsed() >= Duration::from_millis(config.delay_ms) {
                    let eng = Arc::clone(&engine);
                    let tick_result =
                        tokio::task::spawn_blocking(move || eng.lock().unwrap().tick()).await;

                    match tick_result {
                        Ok(Ok(tick)) => {
                            last_spam = Instant::now();
                            cycle_count = tick.cycle_count;
                            let should_log = cycle_count == 1
                                || cycle_count.saturating_sub(last_log_cycle) >= 100;
                            if should_log {
                                last_log_cycle = cycle_count;
                                emit_tool_log_opt(
                                    Some(&app),
                                    format!("[Spammer] cycle #{cycle_count} F1 + click"),
                                );
                            }
                        }
                        Ok(Err(e)) => {
                            let err_msg = e.to_string();
                            recover_ydotool_on_error(
                                &app,
                                &gateway,
                                ydotoold.as_ref(),
                                &mut last_ydotool_recovery,
                                &err_msg,
                            )
                            .await;
                            emit_status_if_changed(
                                &app,
                                &status_arc,
                                SpammerStatusEvent {
                                    active: true,
                                    armed: true,
                                    spamming,
                                    key: "F1".to_string(),
                                    delay_ms: config.delay_ms,
                                    cycle_count,
                                    error: Some(err_msg),
                                },
                            );
                            continue 'main;
                        }
                        Err(e) => {
                            emit_tool_log_opt(
                                Some(&app),
                                format!("[Spammer] ERROR tick (join): {e}"),
                            );
                        }
                    }
                }

                emit_status_if_changed(
                    &app,
                    &status_arc,
                    SpammerStatusEvent {
                        active: true,
                        armed: true,
                        spamming,
                        key: "F1".to_string(),
                        delay_ms: config.delay_ms,
                        cycle_count,
                        error: None,
                    },
                );
            }
            _ = &mut ready_timeout, if !ready_received => {
                let msg = "[Spammer] ro-inputd no respondió (timeout)".to_string();
                emit_tool_log_opt(Some(&app), &msg);
                emit_status_if_changed(
                    &app,
                    &status_arc,
                    SpammerStatusEvent {
                        active: false,
                        error: Some(msg),
                        ..SpammerStatusEvent::default()
                    },
                );
                break 'main;
            }
            changed = stop_rx.changed() => {
                if changed.is_ok() && *stop_rx.borrow() {
                    break 'main;
                }
            }
        }
    }

    let _ = stdin.write_all(b"{\"type\":\"stop\"}\n").await;
    let _ = stdin.flush().await;
    drop(stdin);
    let _ = tokio::time::timeout(Duration::from_secs(2), child.wait()).await;

    emit_tool_log_opt(Some(&app), "[Spammer] Loop detenido");
    emit_status_if_changed(
        &app,
        &status_arc,
        SpammerStatusEvent {
            active: false,
            armed: false,
            spamming: false,
            key: "F1".to_string(),
            delay_ms: config.delay_ms,
            ..SpammerStatusEvent::default()
        },
    );
}

fn spammer_poll_ticker() -> tokio::time::Interval {
    let mut ticker = interval(Duration::from_millis(KEY_POLL_MS));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
    ticker
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ready_message() {
        let line = r#"{"type":"ready","devices":["AT keyboard"],"name":"AT keyboard"}"#;
        assert!(matches!(parse_line(line), Some(InputdMsg::Ready)));
    }

    #[test]
    fn parse_trigger_held_messages() {
        assert!(matches!(
            parse_line(r#"{"type":"trigger","held":true}"#),
            Some(InputdMsg::TriggerHeld(true))
        ));
        assert!(matches!(
            parse_line(r#"{"type":"trigger","held":false}"#),
            Some(InputdMsg::TriggerHeld(false))
        ));
    }

    #[test]
    fn parse_fatal_message() {
        match parse_line(r#"{"type":"fatal","message":"grab failed"}"#) {
            Some(InputdMsg::Fatal(msg)) => assert_eq!(msg, "grab failed"),
            other => panic!("expected fatal, got {other:?}"),
        }
    }

    #[test]
    fn parse_ignores_unknown_or_malformed() {
        assert!(parse_line("not json").is_none());
        assert!(parse_line(r#"{"type":"shutdown"}"#).is_none());
        assert!(parse_line(r#"{"type":"trigger"}"#).is_none());
    }
}
