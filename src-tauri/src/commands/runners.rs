use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

// ── Runner resolution ─────────────────────────────────────────────────────────

pub struct ResolvedRunner {
    pub wine_bin: String,
    pub ld_library_path: Option<String>,
}

pub fn resolve_runner(runner_path: &str) -> Result<ResolvedRunner, String> {
    let path = Path::new(runner_path);

    if path.file_name().and_then(|name| name.to_str()) == Some("proton") {
        let proton_dir = path
            .parent()
            .ok_or_else(|| format!("Ruta Proton inválida: {runner_path}"))?;
        let files_dir = proton_dir.join("files");
        let wine_bin = files_dir.join("bin").join("wine");

        if !wine_bin.exists() {
            return Err(format!(
                "No se encontró el wine de Proton en {}",
                wine_bin.display()
            ));
        }

        let lib64 = files_dir.join("lib64");
        let lib = files_dir.join("lib");
        let ld_library_path = format!("{}:{}", lib64.display(), lib.display());

        return Ok(ResolvedRunner {
            wine_bin: wine_bin.to_string_lossy().to_string(),
            ld_library_path: Some(ld_library_path),
        });
    }

    if !path.exists() {
        return Err(format!("Runner no encontrado: {runner_path}"));
    }

    Ok(ResolvedRunner {
        wine_bin: runner_path.to_string(),
        ld_library_path: None,
    })
}

pub fn canonical_runner_path(path: &Path) -> Option<PathBuf> {
    std::fs::canonicalize(path).ok()
}

// ── Tauri command ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct RunnerInfo {
    pub id: String,
    pub name: String,
    pub path: String,
}

#[tauri::command]
pub async fn list_runners() -> Result<Vec<RunnerInfo>, String> {
    let mut runners = vec![];
    let mut seen = HashSet::new();

    let home = std::env::var("HOME").unwrap_or_default();

    // Proton first — recommended for Gepard Shield clients
    for search_dir in &[
        "/usr/share/steam/compatibilitytools.d".to_string(),
        format!("{}/.local/share/Steam/compatibilitytools.d", home),
        format!("{}/.steam/root/compatibilitytools.d", home),
    ] {
        let dir = Path::new(search_dir);
        if !dir.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(dir) {
            let mut proton_entries: Vec<_> = entries.flatten().collect();
            proton_entries.sort_by_key(|e| e.file_name());
            for entry in proton_entries {
                let proton_bin = entry.path().join("proton");
                if !proton_bin.exists() {
                    continue;
                }
                let Some(canonical) = canonical_runner_path(&proton_bin) else {
                    continue;
                };
                if !seen.insert(canonical.to_string_lossy().to_string()) {
                    continue;
                }

                let tool_name = entry.file_name().to_string_lossy().to_string();
                let label = if tool_name == "proton-cachyos-slr" {
                    format!("{tool_name} (recomendado Gepard)")
                } else {
                    tool_name
                };

                runners.push(RunnerInfo {
                    id: format!("proton-{}", entry.file_name().to_string_lossy()),
                    name: label,
                    path: proton_bin.to_string_lossy().to_string(),
                });
            }
        }
    }

    // System wine binaries (fallback)
    for (path, label) in &[
        ("/usr/bin/wine-cachyos", "Wine CachyOS"),
        ("/usr/bin/wine", "Wine"),
        ("/usr/bin/wine64", "Wine64"),
    ] {
        if !Path::new(path).exists() {
            continue;
        }
        let Some(canonical) = canonical_runner_path(Path::new(path)) else {
            continue;
        };
        if !seen.insert(canonical.to_string_lossy().to_string()) {
            continue;
        }

        runners.push(RunnerInfo {
            id: Path::new(path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            name: format!("{} ({})", label, path),
            path: path.to_string(),
        });
    }

    runners.sort_by(|a, b| runner_sort_key(a).cmp(&runner_sort_key(b)));

    Ok(runners)
}

fn runner_sort_key(runner: &RunnerInfo) -> (u8, String) {
    let priority = if runner.id.contains("proton-cachyos-slr") {
        0
    } else if runner.id.starts_with("proton-") {
        1
    } else {
        2
    };
    (priority, runner.name.clone())
}
