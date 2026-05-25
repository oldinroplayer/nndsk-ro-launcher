use tokio::process::Command;

pub fn wine_command(
    wine_bin: &str,
    ld_library_path: Option<&str>,
    exe_path: &str,
    prefix: &str,
    work_dir: &str,
    apply_env: impl FnOnce(&mut Command),
) -> Command {
    let mut cmd = Command::new(wine_bin);
    cmd.arg(exe_path);
    apply_prefix_env(&mut cmd, prefix);
    apply_env(&mut cmd);
    cmd.current_dir(work_dir);
    apply_runner_env(&mut cmd, ld_library_path);
    cmd
}

pub fn apply_prefix_env(cmd: &mut Command, prefix_path: &str) {
    cmd.env("WINEPREFIX", prefix_path).env("WAYLAND_DISPLAY", "");
}

pub fn apply_game_env(cmd: &mut Command) {
    cmd.env("DXVK_ASYNC", "1")
        .env("DXVK_CONFIG", "d3d9.forceSamplerTypeSpecConstants=True")
        .env("WINE_LARGE_ADDRESS_AWARE", "1")
        .env("WINEDLLOVERRIDES", "d3dimm=n,b;ddraw=n,b");
}

pub fn apply_tool_env(cmd: &mut Command, needs_dgvoodoo_overrides: bool) {
    cmd.env("DXVK_ASYNC", "1").env("WINE_LARGE_ADDRESS_AWARE", "1");
    if needs_dgvoodoo_overrides {
        cmd.env("WINEDLLOVERRIDES", "d3dimm=n,b;ddraw=n,b");
    }
}

pub fn pipe_output(cmd: &mut Command) {
    cmd.stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());
}

/// Inyecta Proton LD_LIBRARY_PATH si el runner lo requiere.
pub fn apply_runner_env(cmd: &mut Command, ld_library_path: Option<&str>) {
    if let Some(proton_libs) = ld_library_path {
        let lib_path = match std::env::var("LD_LIBRARY_PATH") {
            Ok(existing) if !existing.is_empty() => format!("{proton_libs}:{existing}"),
            _ => proton_libs.to_string(),
        };
        cmd.env("LD_LIBRARY_PATH", lib_path);
    }
}

pub async fn kill_wineserver(prefix_path: &str) {
    let mut cmd = Command::new("wineserver");
    cmd.arg("-k");
    apply_prefix_env(&mut cmd, prefix_path);
    let _ = cmd.status().await;
}
