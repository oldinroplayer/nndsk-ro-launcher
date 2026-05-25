use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

use crate::utils::find_file_case_insensitive;

pub const TEMPLATE_FILES: &[&str] = &[
    "D3DImm.dll",
    "DDraw.dll",
    "dgVoodoo.conf",
    "dgVoodooCpl.exe",
];

pub const REQUIRED_FILES: &[&str] = &["D3DImm.dll", "DDraw.dll", "dgVoodoo.conf"];

pub fn template_dir(app: &AppHandle) -> Result<PathBuf, String> {
    if let Ok(resource_dir) = app.path().resource_dir() {
        let bundled = resource_dir.join("dgvoodoo");
        if template_is_complete(&bundled) {
            return Ok(bundled);
        }
    }

    let dev = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("resources/dgvoodoo");
    if template_is_complete(&dev) {
        return Ok(dev);
    }

    Err("Plantilla dgVoodoo no encontrada en el launcher".to_string())
}

pub fn template_is_complete(dir: &Path) -> bool {
    dir.is_dir()
        && TEMPLATE_FILES
            .iter()
            .all(|file| dir.join(file).is_file())
}

pub fn install_files(app: &AppHandle, game_dir: &Path) -> Result<Vec<String>, String> {
    let template_dir = template_dir(app)?;
    let mut installed = Vec::new();

    for file in TEMPLATE_FILES {
        if find_file_case_insensitive(game_dir, file).is_some() {
            continue;
        }

        let source = template_dir.join(file);
        let dest = game_dir.join(file);
        std::fs::copy(&source, &dest)
            .map_err(|e| format!("No se pudo copiar {file}: {e}"))?;
        installed.push((*file).to_string());
    }

    Ok(installed)
}

pub fn uninstall_files(game_dir: &Path) -> Result<Vec<String>, String> {
    let mut removed = Vec::new();

    for file in TEMPLATE_FILES {
        let Some(path) = find_file_case_insensitive(game_dir, file) else {
            continue;
        };

        std::fs::remove_file(&path).map_err(|e| {
            format!(
                "No se pudo eliminar {}: {e}",
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(file)
            )
        })?;
        removed.push(
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(file)
                .to_string(),
        );
    }

    if removed.is_empty() {
        return Err("No hay archivos dgVoodoo para desinstalar".to_string());
    }

    Ok(removed)
}

pub fn validate_conf(content: &str, issues: &mut Vec<String>) {
    if conf_value_anywhere(content, "Version").is_none() {
        issues.push("dgVoodoo.conf no parece válido (falta Version)".to_string());
    }

    if let Some(output_api) = conf_value(content, "General", "OutputAPI") {
        let api = output_api.to_ascii_lowercase();
        if api.is_empty() || api == "disabled" {
            issues.push("OutputAPI no está configurado en dgVoodoo.conf".to_string());
        }
    } else {
        issues.push("OutputAPI no definido en dgVoodoo.conf".to_string());
    }

    if let Some(pass_through) = conf_value(content, "DirectX", "DisableAndPassThru") {
        if pass_through.eq_ignore_ascii_case("true") {
            issues.push(
                "DisableAndPassThru está activo — dgVoodoo no interceptará DirectX".to_string(),
            );
        }
    }
}

fn conf_value_anywhere(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('[') {
            continue;
        }
        if let Some((k, v)) = line.split_once('=') {
            if k.trim().eq_ignore_ascii_case(key) {
                return Some(v.trim().to_string());
            }
        }
    }
    None
}

fn conf_value(content: &str, section: &str, key: &str) -> Option<String> {
    let mut in_section = false;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            let current = &line[1..line.len() - 1];
            in_section = current.eq_ignore_ascii_case(section);
            continue;
        }
        if in_section {
            if let Some((k, v)) = line.split_once('=') {
                if k.trim().eq_ignore_ascii_case(key) {
                    return Some(v.trim().to_string());
                }
            }
        }
    }
    None
}
