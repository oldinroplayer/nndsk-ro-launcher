use std::path::{Path, PathBuf};

use super::paths::app_data_dir;

pub const PREFIX_MARKER: &str = ".ro-launcher-configured";

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

pub fn is_dxvk_installed(prefix_path: &str) -> bool {
    Path::new(prefix_path)
        .join("drive_c/windows/system32/d3d9.dll")
        .exists()
}
