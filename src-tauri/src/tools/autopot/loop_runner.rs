use std::sync::{Arc, Mutex};
use std::time::Instant;

use ro_tools_core::{AutopotConfig, AutopotEngine, CombatInputBackend};
use tauri::AppHandle;
use tokio::sync::watch;

use crate::models::autopot::AutopotStatusEvent;
use crate::tools::input::{
    emit_status_if_changed, recover_ydotool_on_error, InputGateway, InputSource, YdotoolDaemon,
};
use crate::utils::EVENT_AUTOPOT_STATUS;

use super::service::new_ticker;

pub struct RunContext {
    pub app: AppHandle,
    pub memory: ro_tools_linux::ProcMemoryReader,
    pub writer: crate::tools::input::GatewayWriter,
    pub config: AutopotConfig,
    pub profile: ro_tools_core::ClientProfile,
    pub stop_rx: watch::Receiver<bool>,
    pub config_rx: watch::Receiver<AutopotConfig>,
    pub status_arc: Arc<Mutex<AutopotStatusEvent>>,
    pub gateway: InputGateway,
    pub ydotoold: Arc<YdotoolDaemon>,
    pub backend: CombatInputBackend,
}

pub async fn run(context: RunContext) {
    let RunContext {
        app,
        memory,
        writer,
        config,
        profile,
        mut stop_rx,
        mut config_rx,
        status_arc,
        gateway,
        ydotoold,
        backend,
    } = context;
    let min_delay_ms = if backend == CombatInputBackend::Uinput {
        10
    } else {
        50
    };
    let engine = Arc::new(Mutex::new(AutopotEngine::new_with_min_delay(
        memory,
        writer,
        config.clone(),
        profile,
        min_delay_ms,
    )));
    let mut current_config = config;
    let mut ticker = new_ticker(current_config.delay_ms, backend);
    let mut metrics_ticker = tokio::time::interval(std::time::Duration::from_secs(10));
    metrics_ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    metrics_ticker.tick().await;
    let mut scan_periods_us = Vec::new();
    let mut scan_durations_us = Vec::new();
    let mut last_scan: Option<Instant> = None;
    let mut terminal_error: Option<String> = None;
    let mut tick_count: u64 = 0;
    let mut last_ydotool_recovery = Instant::now()
        .checked_sub(std::time::Duration::from_secs(10))
        .unwrap_or_else(Instant::now);

    loop {
        tokio::select! {
            _ = ticker.tick() => {
                tick_count += 1;
                let scan_started = Instant::now();
                if let Some(previous) = last_scan.replace(scan_started) {
                    scan_periods_us.push(scan_started.duration_since(previous).as_micros() as u64);
                }
                let tick_result = if backend == CombatInputBackend::Uinput {
                    engine.lock().unwrap().tick().map_err(|error| error.to_string())
                } else {
                    let engine = Arc::clone(&engine);
                    match tokio::task::spawn_blocking(move || engine.lock().unwrap().tick()).await {
                        Ok(result) => result.map_err(|error| error.to_string()),
                        Err(error) => Err(format!("join: {error}")),
                    }
                };
                scan_durations_us.push(scan_started.elapsed().as_micros() as u64);

                match tick_result {
                    Ok(tick) => {
                        // Proactive pulses run at the regular AutoPot cadence; omit them from
                        // the event log so the Logs panel remains useful during long sessions.
                        if tick.potted_hp || tick.potted_sp {
                            crate::utils::emit_tool_log_opt(
                                Some(&app),
                                format!(
                                    "[AutoPot] tick#{tick_count} HP={} SP={} proactivo={} | {}/{} HP · {}/{} SP · '{}'",
                                    if tick.potted_hp { "sí" } else { "—" },
                                    if tick.potted_sp { "sí" } else { "—" },
                                    if tick.proactive_hp_pulse { "sí" } else { "—" },
                                    tick.cur_hp, tick.max_hp,
                                    tick.cur_sp, tick.max_sp,
                                    tick.character_name,
                                ),
                            );
                        }
                        emit_status_if_changed(
                            &app,
                            &status_arc,
                            EVENT_AUTOPOT_STATUS,
                            AutopotStatusEvent {
                                active: true,
                                input_backend: backend,
                                effective_delay_ms: current_config.delay_ms,
                                cur_hp: tick.cur_hp,
                                max_hp: tick.max_hp,
                                cur_sp: tick.cur_sp,
                                max_sp: tick.max_sp,
                                hp_percent: current_config.hp_percent,
                                sp_percent: current_config.sp_percent,
                                character_name: tick.character_name,
                                error: None,
                            },
                        );
                    }
                    Err(err_msg) => {
                        if backend == CombatInputBackend::Ydotool {
                            recover_ydotool_on_error(
                                &app,
                                &gateway,
                                ydotoold.as_ref(),
                                &mut last_ydotool_recovery,
                                &err_msg,
                                "[Input] ydotoold recuperado",
                            )
                            .await;
                        }

                        let prev = status_arc.lock().unwrap().clone();
                        emit_status_if_changed(
                            &app,
                            &status_arc,
                            EVENT_AUTOPOT_STATUS,
                            AutopotStatusEvent {
                                active: true,
                                input_backend: backend,
                                effective_delay_ms: current_config.delay_ms,
                                cur_hp: prev.cur_hp,
                                max_hp: prev.max_hp,
                                cur_sp: prev.cur_sp,
                                max_sp: prev.max_sp,
                                character_name: prev.character_name,
                                error: Some(err_msg.clone()),
                                hp_percent: current_config.hp_percent,
                                sp_percent: current_config.sp_percent,
                            },
                        );
                        crate::utils::emit_tool_log_opt(
                            Some(&app),
                            format!("[AutoPot] ERROR tick: {err_msg}"),
                        );
                        if backend == CombatInputBackend::Uinput {
                            terminal_error = Some(err_msg);
                            break;
                        }
                    }
                }
            }
            changed = config_rx.changed() => {
                if changed.is_ok() {
                    current_config = config_rx
                        .borrow()
                        .clone()
                        .clamped_with_min_delay(min_delay_ms);
                    engine
                        .lock()
                        .unwrap()
                        .update_config_with_min_delay(current_config.clone(), min_delay_ms);
                    ticker = new_ticker(current_config.delay_ms, backend);
                    crate::utils::emit_tool_log_opt(
                        Some(&app),
                        format!(
                            "[AutoPot] Config actualizada HP={}% SP={}% delay={}ms proactivo={}",
                            current_config.hp_percent,
                            current_config.sp_percent,
                            current_config.delay_ms,
                            if current_config.proactive_mode { "sí" } else { "no" },
                        ),
                    );
                }
            }
            _ = metrics_ticker.tick() => {
                log_metrics(
                    &app,
                    &gateway,
                    backend,
                    current_config.delay_ms,
                    &mut scan_periods_us,
                    &mut scan_durations_us,
                    false,
                );
            }
            changed = stop_rx.changed() => {
                if changed.is_ok() && *stop_rx.borrow() {
                    break;
                }
            }
        }
    }

    log_metrics(
        &app,
        &gateway,
        backend,
        current_config.delay_ms,
        &mut scan_periods_us,
        &mut scan_durations_us,
        true,
    );
    crate::utils::emit_tool_log_opt(Some(&app), "[AutoPot] Loop detenido");
    let idle = AutopotStatusEvent {
        active: false,
        input_backend: backend,
        effective_delay_ms: current_config.delay_ms,
        hp_percent: current_config.hp_percent,
        sp_percent: current_config.sp_percent,
        error: terminal_error,
        ..AutopotStatusEvent::default()
    };
    emit_status_if_changed(&app, &status_arc, EVENT_AUTOPOT_STATUS, idle);
}

fn log_metrics(
    app: &AppHandle,
    gateway: &InputGateway,
    backend: CombatInputBackend,
    effective_delay_ms: u64,
    scan_periods_us: &mut Vec<u64>,
    scan_durations_us: &mut Vec<u64>,
    final_window: bool,
) {
    let window = if final_window { "final" } else { "10s" };
    let line = if backend == CombatInputBackend::Uinput {
        let input = gateway.uinput_metrics(InputSource::Autopot);
        format!(
            "{} effective_delay_ms={} autopot_read_period_us[p50/p95/p99]={}/{}/{} scan_duration_us[p50/p95/p99]={}/{}/{}",
            input.log_line(InputSource::Autopot, final_window),
            effective_delay_ms,
            percentile(scan_periods_us, 50),
            percentile(scan_periods_us, 95),
            percentile(scan_periods_us, 99),
            percentile(scan_durations_us, 50),
            percentile(scan_durations_us, 95),
            percentile(scan_durations_us, 99),
        )
    } else {
        format!(
            "[input-metrics] backend=ydotool source=autopot window={window} effective_delay_ms={effective_delay_ms} samples={} period_us[p50/p95/p99]={}/{}/{} queue_us=not_applicable cycle_us[p50/p95/p99]={}/{}/{} overruns=0 dropped=0 input_errors=0",
            scan_durations_us.len(),
            percentile(scan_periods_us, 50),
            percentile(scan_periods_us, 95),
            percentile(scan_periods_us, 99),
            percentile(scan_durations_us, 50),
            percentile(scan_durations_us, 95),
            percentile(scan_durations_us, 99),
        )
    };
    crate::utils::emit_tool_log_opt(Some(app), line);
    scan_periods_us.clear();
    scan_durations_us.clear();
}

fn percentile(values: &[u64], percent: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    sorted[((sorted.len() - 1) * percent).div_ceil(100)]
}
