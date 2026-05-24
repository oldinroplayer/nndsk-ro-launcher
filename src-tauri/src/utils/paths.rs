use std::path::{Path, PathBuf};

pub const PREFIX_MARKER: &str = ".ro-launcher-configured";

pub fn app_data_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(format!("{home}/.local/share/ro-launcher"))
}

pub fn prefix_path() -> String {
    app_data_dir()
        .join("prefix")
        .to_string_lossy()
        .to_string()
}

pub fn effective_prefix(wine_prefix: Option<String>) -> String {
    wine_prefix.unwrap_or_else(prefix_path)
}

pub fn prefix_marker_path(prefix_path: &str) -> PathBuf {
    Path::new(prefix_path).join(PREFIX_MARKER)
}

pub fn is_prefix_configured(prefix_path: &str) -> bool {
    prefix_marker_path(prefix_path).exists()
}

pub fn write_prefix_marker(prefix_path: &str) -> Result<(), String> {
    std::fs::write(prefix_marker_path(prefix_path), "configured").map_err(|e| e.to_string())
}

pub fn work_dir_from_exe(exe_path: &str) -> String {
    Path::new(exe_path)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

pub fn required_game_dir(exe_path: &str) -> Result<String, String> {
    let dir = work_dir_from_exe(exe_path);
    if dir.is_empty() {
        Err("Ruta del ejecutable inválida".to_string())
    } else {
        Ok(dir)
    }
}

pub fn is_dxvk_installed(prefix_path: &str) -> bool {
    Path::new(prefix_path)
        .join("drive_c/windows/system32/d3d9.dll")
        .exists()
}
