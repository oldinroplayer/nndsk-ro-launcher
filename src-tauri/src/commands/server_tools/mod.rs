mod dgvoodoo;
mod scan;

use tauri::AppHandle;

use crate::utils::resolve_effective_runner;
use crate::models::server::ServerConfig;
use crate::models::server_tools::{
    InstallDgVoodooResult, ServerToolsStatus, UninstallDgVoodooResult,
};
use crate::models::tool_kind::ToolKind;
use crate::utils::{
    apply_tool_env, effective_prefix, required_game_dir, wine_command,
};

#[tauri::command]
pub async fn scan_server_tools(
    app: AppHandle,
    server: ServerConfig,
) -> Result<ServerToolsStatus, String> {
    scan_server_status(&app, &server)
}

#[tauri::command]
pub async fn install_dgvoodoo(
    app: AppHandle,
    server: ServerConfig,
) -> Result<InstallDgVoodooResult, String> {
    let game_dir = required_game_dir(&server.executable_path)?;
    let installed = dgvoodoo::install_files(&app, std::path::Path::new(&game_dir))?;
    let status = scan_server_status(&app, &server)?;
    Ok(InstallDgVoodooResult { installed, status })
}

#[tauri::command]
pub async fn uninstall_dgvoodoo(
    app: AppHandle,
    server: ServerConfig,
) -> Result<UninstallDgVoodooResult, String> {
    let game_dir = required_game_dir(&server.executable_path)?;
    let removed = dgvoodoo::uninstall_files(std::path::Path::new(&game_dir))?;
    let status = scan_server_status(&app, &server)?;
    Ok(UninstallDgVoodooResult { removed, status })
}

#[tauri::command]
pub async fn launch_server_tool(
    app: AppHandle,
    server: ServerConfig,
    tool: ToolKind,
) -> Result<(), String> {
    let status = scan_server_status(&app, &server)?;
    let exe_path = match tool {
        ToolKind::OpenSetup => status
            .open_setup
            .path
            .ok_or_else(|| "OpenSetup no encontrado".to_string())?,
        ToolKind::Patcher => status
            .patcher
            .path
            .ok_or_else(|| "Patcher no encontrado".to_string())?,
        ToolKind::DgVoodoo => status
            .dgvoodoo
            .cpl
            .path
            .ok_or_else(|| "dgVoodoo Control Panel no encontrado".to_string())?,
    };

    let prefix = effective_prefix(server.wine_prefix.clone());
    let resolved = resolve_effective_runner(server.runner.clone()).await?;

    let work_dir = required_game_dir(&exe_path)
        .or_else(|_| required_game_dir(&server.executable_path))?;

    let mut cmd = wine_command(
        &resolved.wine_bin,
        resolved.ld_library_path.as_deref(),
        &exe_path,
        &prefix,
        &work_dir,
        |cmd| apply_tool_env(cmd, tool.needs_dgvoodoo_overrides()),
    );

    cmd.spawn()
        .map_err(|e| format!("Error al abrir la herramienta: {e}"))?;

    Ok(())
}

fn scan_server_status(
    app: &AppHandle,
    server: &ServerConfig,
) -> Result<ServerToolsStatus, String> {
    let game_dir = required_game_dir(&server.executable_path)?;
    let can_auto_install = dgvoodoo::template_dir(app).is_ok();
    scan::scan_game_dir(&game_dir, server, can_auto_install)
}
