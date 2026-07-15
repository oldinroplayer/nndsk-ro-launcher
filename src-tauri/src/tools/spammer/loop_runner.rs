use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ro_tools_core::{CombatInputBackend, HeldKeyWriter, SpammerConfig, SpammerEngine};
use tauri::AppHandle;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::watch;
use tokio::time::{interval, MissedTickBehavior};

use super::gear::{self, GearMode};
use crate::models::spammer::SpammerStatusEvent;
use crate::tools::input::{
    emit_status_if_changed, recover_ydotool_on_error, InputGateway, InputSource, YdotoolDaemon,
};
use crate::utils::{emit_tool_log_opt, EVENT_SPAMMER_STATUS};

const STABLE_KEY_POLL_MS: u64 = 5;
const INPUTD_READY_TIMEOUT_SECS: u64 = 5;

#[derive(Debug)]
enum InputdMsg {
    Ready,
    TriggerHeld { key: String, held: bool },
    Fatal(String),
}

fn parse_line(line: &str) -> Option<InputdMsg> {
    let v: serde_json::Value = serde_json::from_str(line).ok()?;
    match v.get("type")?.as_str()? {
        "ready" => Some(InputdMsg::Ready),
        "trigger" => {
            let key = v.get("key")?.as_str()?.to_string();
            Some(InputdMsg::TriggerHeld {
                key,
                held: v.get("held")?.as_bool()?,
            })
        }
        "fatal" => Some(InputdMsg::Fatal(
            v.get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("fatal")
                .to_string(),
        )),
        _ => None,
    }
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

fn build_status(
    config: &SpammerConfig,
    backend: CombatInputBackend,
    effective_delay_ms: u64,
    active_key: &str,
    cycle_count: u64,
    error: Option<String>,
    active: bool,
    armed: bool,
    gear_mode: Option<&str>,
) -> SpammerStatusEvent {
    SpammerStatusEvent {
        active,
        input_backend: backend,
        effective_delay_ms,
        armed,
        spamming: !active_key.is_empty(),
        key: active_key.to_string(),
        delay_ms: config.delay_ms,
        cycle_count,
        error,
        gear_mode: gear_mode.map(str::to_string),
    }
}

pub async fn run(
    app: AppHandle,
    writer: crate::tools::input::GatewayWriter,
    config: SpammerConfig,
    backend: CombatInputBackend,
    mut stop_rx: watch::Receiver<bool>,
    status_arc: Arc<Mutex<SpammerStatusEvent>>,
    gateway: InputGateway,
    ydotoold: Arc<YdotoolDaemon>,
) {
    let config = config.clamped();
    let effective_delay_ms = if backend == CombatInputBackend::Uinput {
        config.delay_ms.max(10)
    } else {
        config.delay_ms
    };
    let triggers_arg = config.keys.join(",");

    let inputd_path = find_ro_inputd();

    let mut child = match tokio::process::Command::new(&inputd_path)
        .arg("--triggers")
        .arg(&triggers_arg)
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
                EVENT_SPAMMER_STATUS,
                build_status(
                    &config,
                    backend,
                    effective_delay_ms,
                    "",
                    0,
                    Some(msg),
                    false,
                    false,
                    None,
                ),
            );
            return;
        }
    };

    let mut stdin = child.stdin.take().expect("stdin piped");
    let mut lines = BufReader::new(child.stdout.take().expect("stdout piped")).lines();

    emit_tool_log_opt(
        Some(&app),
        format!("[Spammer] Esperando ro-inputd (triggers: {triggers_arg})..."),
    );

    let cleanup_writer = writer.clone();
    let engine = Arc::new(Mutex::new(SpammerEngine::new(writer, config.clone())));
    let mut poll = spammer_poll_ticker(backend, effective_delay_ms);
    let mut metrics_ticker = interval(Duration::from_secs(10));
    metrics_ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
    metrics_ticker.tick().await;
    let mut last_spam = Instant::now()
        .checked_sub(Duration::from_secs(1))
        .unwrap_or_else(Instant::now);
    let mut last_log_cycle: u64 = 0;
    let mut cycle_count: u64 = 0;
    let mut active_key = String::new();
    let mut held_keys: Vec<String> = Vec::new();
    let mut gear_mode: Option<&'static str> = None;
    let mut ready_received = false;
    let mut terminal_error: Option<String> = None;
    let mut last_ydotool_recovery = Instant::now()
        .checked_sub(Duration::from_secs(10))
        .unwrap_or_else(Instant::now);
    let mut last_status_emit = Instant::now()
        .checked_sub(Duration::from_secs(1))
        .unwrap_or_else(Instant::now);
    let mut cycle_periods_us = Vec::new();
    let mut cycle_durations_us = Vec::new();
    let mut last_completed_cycle: Option<Instant> = None;
    let mut ydotool_input_errors = 0u64;

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
                            for key in &config.keys {
                                let _ = cleanup_writer.key_up(key);
                            }
                            emit_status_if_changed(
                                &app,
                                &status_arc,
                                EVENT_SPAMMER_STATUS,
                                build_status(&config, backend, effective_delay_ms, "", 0, None, true, true, gear_mode),
                            );
                        }
                        Some(InputdMsg::TriggerHeld { key, held }) if ready_received => {
                            let was_held = held_keys.iter().any(|k| k == &key);

                            if held {
                                if !was_held {
                                    held_keys.push(key.clone());
                                }
                                let was_active = !active_key.is_empty();
                                active_key = key.clone();
                                if !was_active {
                                    cycle_count = 0;
                                    last_log_cycle = 0;
                                    last_spam = Instant::now()
                                        .checked_sub(Duration::from_millis(config.delay_ms))
                                        .unwrap_or_else(Instant::now);
                                    emit_tool_log_opt(
                                        Some(&app),
                                        format!("[Spammer] Spam activo ({key})"),
                                    );
                                    if backend == CombatInputBackend::Uinput {
                                        poll.reset_immediately();
                                    }
                                }
                            } else {
                                held_keys.retain(|k| k != &key);
                                if active_key == key {
                                    if let Some(next) = held_keys.last() {
                                        active_key = next.clone();
                                        cycle_count = 0;
                                        last_log_cycle = 0;
                                    } else {
                                        active_key.clear();
                                    }
                                }
                            }

                            // Gear switch: cada regla es edge-triggered por su propia tecla.
                            // Press fresco → equipa ATK; release de una tecla presionada → DEF.
                            let fresh_press = held && !was_held;
                            let fresh_release = !held && was_held;
                            if config.gear_switch.enabled && (fresh_press || fresh_release) {
                                if let Some(rule) = config.gear_switch.rule_for(&key) {
                                    let (keys, mode, label) = if fresh_press {
                                        (rule.atk_keys.clone(), GearMode::Atk, "ATK")
                                    } else {
                                        (rule.def_keys.clone(), GearMode::Def, "DEF")
                                    };
                                    if !keys.is_empty() {
                                        let switch_delay = config.gear_switch.switch_delay_ms;
                                        let writer = match gateway.writer_for(
                                            backend,
                                            InputSource::Gear,
                                            switch_delay,
                                        ) {
                                            Ok(writer) => writer,
                                            Err(error) => {
                                                emit_tool_log_opt(
                                                    Some(&app),
                                                    format!("[Spammer] ERROR gear input: {error}"),
                                                );
                                                continue 'main;
                                            }
                                        };
                                        let keys_log = keys.join("+");
                                        let equip_result = tokio::task::spawn_blocking(
                                            move || gear::equip(&writer, &keys, switch_delay),
                                        )
                                        .await;
                                        match equip_result {
                                            Ok(Ok(())) => {
                                                gear_mode = Some(mode.as_str());
                                                emit_tool_log_opt(
                                                    Some(&app),
                                                    format!(
                                                        "[Spammer] Gear {label} {key}→{keys_log}"
                                                    ),
                                                );
                                            }
                                            Ok(Err(e)) => {
                                                let err_msg = e.to_string();
                                                if backend == CombatInputBackend::Ydotool {
                                                    recover_ydotool_on_error(
                                                        &app,
                                                        &gateway,
                                                        ydotoold.as_ref(),
                                                        &mut last_ydotool_recovery,
                                                        err_msg.as_str(),
                                                        "[Input] ydotoold recuperado (gear)",
                                                    )
                                                    .await;
                                                }
                                            }
                                            Err(e) => {
                                                emit_tool_log_opt(
                                                    Some(&app),
                                                    format!("[Spammer] ERROR gear (join): {e}"),
                                                );
                                            }
                                        }
                                    } else {
                                        gear_mode = Some(mode.as_str());
                                    }
                                }
                            }
                        }
                        Some(InputdMsg::Fatal(msg)) => {
                            emit_tool_log_opt(Some(&app), format!("[Spammer] Fatal ro-inputd: {msg}"));
                            emit_status_if_changed(
                                &app,
                                &status_arc,
                                EVENT_SPAMMER_STATUS,
                                build_status(
                                    &config,
                                    backend,
                                    effective_delay_ms,
                                    &active_key,
                                    cycle_count,
                                    Some(msg),
                                    false,
                                    false,
                                    None,
                                ),
                            );
                            break 'main;
                        }
                        _ => {}
                    }
                    _ => {
                        let msg = "[Spammer] ro-inputd terminó inesperadamente".to_string();
                        emit_tool_log_opt(Some(&app), &msg);
                        emit_status_if_changed(
                            &app,
                            &status_arc,
                            EVENT_SPAMMER_STATUS,
                            build_status(
                                &config,
                                backend,
                                effective_delay_ms,
                                &active_key,
                                cycle_count,
                                Some(msg),
                                false,
                                false,
                                None,
                            ),
                        );
                        break 'main;
                    }
                }
            }
            scheduled = poll.tick(), if ready_received => {
                let cycle_due = backend == CombatInputBackend::Uinput
                    || last_spam.elapsed() >= Duration::from_millis(config.delay_ms);
                if !active_key.is_empty() && cycle_due {
                    let tick_key = active_key.clone();
                    let log_key = tick_key.clone();
                    let cycle_started = Instant::now();
                    let tick_result = if backend == CombatInputBackend::Uinput {
                        let deadline = scheduled.into_std() + Duration::from_millis(effective_delay_ms);
                        engine
                            .lock()
                            .unwrap()
                            .tick_with_deadline(&tick_key, Some(deadline))
                            .map_err(|error| error.to_string())
                    } else {
                        let eng = Arc::clone(&engine);
                        match tokio::task::spawn_blocking(move || eng.lock().unwrap().tick(&tick_key)).await {
                            Ok(result) => result.map_err(|error| error.to_string()),
                            Err(error) => Err(format!("join: {error}")),
                        }
                    };

                    match tick_result {
                        Ok(tick) if tick.cycled => {
                            if let Some(previous) = last_completed_cycle.replace(cycle_started) {
                                cycle_periods_us.push(
                                    cycle_started.duration_since(previous).as_micros() as u64,
                                );
                            }
                            cycle_durations_us.push(cycle_started.elapsed().as_micros() as u64);
                            last_spam = Instant::now();
                            cycle_count += 1;
                            let should_log = cycle_count == 1
                                || cycle_count.saturating_sub(last_log_cycle) >= 100;
                            if should_log {
                                last_log_cycle = cycle_count;
                                emit_tool_log_opt(
                                    Some(&app),
                                    format!("[Spammer] cycle #{cycle_count} {log_key} + click"),
                                );
                            }
                        }
                        Err(err_msg) => {
                            if backend == CombatInputBackend::Ydotool {
                                ydotool_input_errors += 1;
                            }
                            if backend == CombatInputBackend::Ydotool {
                                recover_ydotool_on_error(
                                    &app,
                                    &gateway,
                                    ydotoold.as_ref(),
                                    &mut last_ydotool_recovery,
                                    err_msg.as_str(),
                                    "[Input] ydotoold recuperado (spammer)",
                                )
                                .await;
                            }
                            emit_status_if_changed(
                                &app,
                                &status_arc,
                                EVENT_SPAMMER_STATUS,
                                build_status(
                                    &config,
                                    backend,
                                    effective_delay_ms,
                                    &active_key,
                                    cycle_count,
                                    Some(err_msg.clone()),
                                    true,
                                    true,
                                    gear_mode,
                                ),
                            );
                            if backend == CombatInputBackend::Uinput {
                                terminal_error = Some(err_msg);
                                break 'main;
                            }
                            continue 'main;
                        }
                        _ => {}
                    }
                }

                if last_status_emit.elapsed() >= Duration::from_millis(250) {
                    last_status_emit = Instant::now();
                    emit_status_if_changed(
                        &app,
                        &status_arc,
                        EVENT_SPAMMER_STATUS,
                        build_status(
                            &config,
                            backend,
                            effective_delay_ms,
                            &active_key,
                            cycle_count,
                            None,
                            true,
                            true,
                            gear_mode,
                        ),
                    );
                }
            }
            _ = metrics_ticker.tick() => {
                log_metrics(
                    &app,
                    &gateway,
                    backend,
                    effective_delay_ms,
                    &mut cycle_periods_us,
                    &mut cycle_durations_us,
                    &mut ydotool_input_errors,
                    false,
                );
            }
            _ = &mut ready_timeout, if !ready_received => {
                let msg = "[Spammer] ro-inputd no respondió (timeout)".to_string();
                emit_tool_log_opt(Some(&app), &msg);
                emit_status_if_changed(
                    &app,
                    &status_arc,
                    EVENT_SPAMMER_STATUS,
                    build_status(
                        &config,
                        backend,
                        effective_delay_ms,
                        &active_key,
                        cycle_count,
                        Some(msg),
                        false,
                        false,
                        None,
                    ),
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

    log_metrics(
        &app,
        &gateway,
        backend,
        effective_delay_ms,
        &mut cycle_periods_us,
        &mut cycle_durations_us,
        &mut ydotool_input_errors,
        true,
    );
    emit_tool_log_opt(Some(&app), "[Spammer] Loop detenido");
    emit_status_if_changed(
        &app,
        &status_arc,
        EVENT_SPAMMER_STATUS,
        build_status(
            &config,
            backend,
            effective_delay_ms,
            "",
            cycle_count,
            terminal_error,
            false,
            false,
            None,
        ),
    );
}

fn log_metrics(
    app: &AppHandle,
    gateway: &InputGateway,
    backend: CombatInputBackend,
    effective_delay_ms: u64,
    periods_us: &mut Vec<u64>,
    durations_us: &mut Vec<u64>,
    ydotool_input_errors: &mut u64,
    final_window: bool,
) {
    let window = if final_window { "final" } else { "10s" };
    let line = if backend == CombatInputBackend::Uinput {
        format!(
            "{} effective_delay_ms={effective_delay_ms}",
            gateway
                .uinput_metrics(InputSource::Spammer)
                .log_line(InputSource::Spammer, final_window)
        )
    } else {
        format!(
            "[input-metrics] backend=ydotool source=spammer window={window} effective_delay_ms={effective_delay_ms} samples={} period_us[p50/p95/p99]={}/{}/{} queue_us=not_applicable cycle_us[p50/p95/p99]={}/{}/{} overruns=0 dropped=0 input_errors={}",
            durations_us.len(),
            percentile(periods_us, 50),
            percentile(periods_us, 95),
            percentile(periods_us, 99),
            percentile(durations_us, 50),
            percentile(durations_us, 95),
            percentile(durations_us, 99),
            *ydotool_input_errors,
        )
    };
    emit_tool_log_opt(Some(app), line);
    periods_us.clear();
    durations_us.clear();
    *ydotool_input_errors = 0;
}

fn percentile(values: &[u64], percent: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[((sorted.len() - 1) * percent).div_ceil(100)]
}

fn spammer_poll_ticker(
    backend: CombatInputBackend,
    effective_delay_ms: u64,
) -> tokio::time::Interval {
    let period = if backend == CombatInputBackend::Uinput {
        effective_delay_ms
    } else {
        STABLE_KEY_POLL_MS
    };
    let mut ticker = interval(Duration::from_millis(period));
    ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
    ticker
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ready_message() {
        let line = r#"{"type":"ready","devices":["AT keyboard"],"name":"AT keyboard","triggers":["F1","F2"]}"#;
        assert!(matches!(parse_line(line), Some(InputdMsg::Ready)));
    }

    #[test]
    fn parse_trigger_held_messages() {
        assert!(matches!(
            parse_line(r#"{"type":"trigger","key":"F2","held":true}"#),
            Some(InputdMsg::TriggerHeld { key, held: true }) if key == "F2"
        ));
        assert!(matches!(
            parse_line(r#"{"type":"trigger","key":"F2","held":false}"#),
            Some(InputdMsg::TriggerHeld { key, held: false }) if key == "F2"
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
        assert!(parse_line(r#"{"type":"trigger","held":true}"#).is_none());
    }
}
