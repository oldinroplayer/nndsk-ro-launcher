use std::path::Path;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_dir_from_unix_path() {
        assert_eq!(
            work_dir_from_exe("/home/user/RO/client.exe"),
            "/home/user/RO"
        );
    }

    #[test]
    fn work_dir_from_wine_drive_path() {
        assert_eq!(
            work_dir_from_exe("/home/user/.wine/drive_c/Games/RO/client.exe"),
            "/home/user/.wine/drive_c/Games/RO"
        );
    }

    #[test]
    fn required_game_dir_rejects_bare_filename() {
        assert!(required_game_dir("client.exe").is_err());
    }

    #[test]
    fn required_game_dir_accepts_valid_path() {
        assert_eq!(required_game_dir("/opt/ro/Ragexe.exe").unwrap(), "/opt/ro");
    }
}
