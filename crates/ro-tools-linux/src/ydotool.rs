use crate::keyboard::key_label_to_keycode;
use ro_tools_core::{HeldKeyWriter, KeyPressWriter, PointerWriter, ToolsError};
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::Mutex;

/// Inter-key delay for autopot full press (ms).
const INPUT_EVENT_DELAY_MS: &str = "2";

/// Ruta del socket que usa ydotoold (XDG_RUNTIME_DIR o /run/user/$UID).
pub fn ydotool_socket_path() -> String {
    if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
        format!("{xdg}/.ydotool_socket")
    } else {
        format!("/run/user/{}/.ydotool_socket", current_uid())
    }
}

pub fn is_ydotool_socket_ready() -> bool {
    Path::new(&ydotool_socket_path()).exists()
}

/// Comprueba que ydotoold responde (no basta con que exista el archivo socket).
pub fn is_ydotool_responsive() -> bool {
    let path = ydotool_socket_path();
    if !Path::new(&path).exists() {
        return false;
    }

    Command::new("ydotool")
        .env("YDOTOOL_SOCKET", &path)
        .arg("type")
        .arg("")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn remove_stale_ydotool_socket() {
    let path = ydotool_socket_path();
    if Path::new(&path).exists() && !is_ydotool_responsive() {
        let _ = std::fs::remove_file(&path);
    }
}

fn binary_on_path(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

pub fn ydotool_installed() -> bool {
    binary_on_path("ydotool")
}

pub fn ydotoold_installed() -> bool {
    binary_on_path("ydotoold")
}

pub fn ydotool_input_installed() -> bool {
    ydotool_installed() && ydotoold_installed()
}

pub fn current_uid() -> u32 {
    unsafe { libc::getuid() }
}

pub fn current_gid() -> u32 {
    unsafe { libc::getgid() }
}

pub struct YdotoolInput {
    socket_path: String,
}

impl YdotoolInput {
    pub fn new() -> Result<Self, ToolsError> {
        let path = ydotool_socket_path();
        if !Path::new(&path).exists() {
            return Err(ToolsError::Other(format!(
                "input virtual no disponible (socket: {path})"
            )));
        }
        Ok(Self { socket_path: path })
    }

    fn run(&self, args: &[&str]) -> Result<(), ToolsError> {
        let status = Command::new("ydotool")
            .env("YDOTOOL_SOCKET", &self.socket_path)
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|e| ToolsError::Other(format!("ydotool: {e}")))?;

        if !status.success() {
            return Err(ToolsError::Other(format!(
                "ydotool falló: {:?}",
                status.code()
            )));
        }
        Ok(())
    }
}

impl KeyPressWriter for YdotoolInput {
    fn press_key(&self, key: &str) -> Result<(), ToolsError> {
        let code = key_to_code(key).ok_or_else(|| ToolsError::Input {
            key: key.to_string(),
            message: "tecla no soportada".into(),
        })?;

        self.run(&[
            "key",
            "-d",
            INPUT_EVENT_DELAY_MS,
            &format!("{code}:1"),
            &format!("{code}:0"),
        ])
        .map_err(|e| ToolsError::Input {
            key: key.to_string(),
            message: e.to_string(),
        })
    }
}

impl PointerWriter for YdotoolInput {
    fn click_left(&self) -> Result<(), ToolsError> {
        // 4RTools: LBUTTONDOWN → 1ms → LBUTTONUP (separado, no 0xC0 atómico)
        self.run(&["click", "-d", "1", "0x40"])
            .map_err(|e| ToolsError::Input {
                key: "click".into(),
                message: e.to_string(),
            })?;
        std::thread::sleep(std::time::Duration::from_millis(1));
        self.run(&["click", "-d", "1", "0x80"])
            .map_err(|e| ToolsError::Input {
                key: "click".into(),
                message: e.to_string(),
            })
    }
}

impl HeldKeyWriter for YdotoolInput {
    fn key_down(&self, key: &str) -> Result<(), ToolsError> {
        let code = key_to_code(key).ok_or_else(|| ToolsError::Input {
            key: key.to_string(),
            message: "tecla no soportada".into(),
        })?;

        self.run(&["key", "-d", "1", &format!("{code}:1")])
            .map_err(|e| ToolsError::Input {
                key: key.to_string(),
                message: e.to_string(),
            })
    }

    fn key_up(&self, key: &str) -> Result<(), ToolsError> {
        let code = key_to_code(key).ok_or_else(|| ToolsError::Input {
            key: key.to_string(),
            message: "tecla no soportada".into(),
        })?;

        self.run(&["key", "-d", "1", &format!("{code}:0")])
            .map_err(|e| ToolsError::Input {
                key: key.to_string(),
                message: e.to_string(),
            })
    }
}

/// Inicializa ydotool sólo cuando una ruta de compatibilidad lo necesita.
pub struct LazyYdotoolInput {
    inner: Mutex<Option<YdotoolInput>>,
}

impl LazyYdotoolInput {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    pub fn reset(&self) {
        if let Ok(mut guard) = self.inner.lock() {
            *guard = None;
        }
    }
}

impl Default for LazyYdotoolInput {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyPressWriter for LazyYdotoolInput {
    fn press_key(&self, key: &str) -> Result<(), ToolsError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| ToolsError::Other("ydotool lock poisoned".into()))?;
        if guard.is_none() {
            *guard = Some(YdotoolInput::new()?);
        }
        guard.as_ref().unwrap().press_key(key)
    }
}

impl PointerWriter for LazyYdotoolInput {
    fn click_left(&self) -> Result<(), ToolsError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| ToolsError::Other("ydotool lock poisoned".into()))?;
        if guard.is_none() {
            *guard = Some(YdotoolInput::new()?);
        }
        guard.as_ref().unwrap().click_left()
    }
}

impl HeldKeyWriter for LazyYdotoolInput {
    fn key_down(&self, key: &str) -> Result<(), ToolsError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| ToolsError::Other("ydotool lock poisoned".into()))?;
        if guard.is_none() {
            *guard = Some(YdotoolInput::new()?);
        }
        guard.as_ref().unwrap().key_down(key)
    }

    fn key_up(&self, key: &str) -> Result<(), ToolsError> {
        let mut guard = self
            .inner
            .lock()
            .map_err(|_| ToolsError::Other("ydotool lock poisoned".into()))?;
        if guard.is_none() {
            *guard = Some(YdotoolInput::new()?);
        }
        guard.as_ref().unwrap().key_up(key)
    }
}

fn key_to_code(key: &str) -> Option<u16> {
    key_label_to_keycode(key).map(|code| code.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_f1_through_f12() {
        let expected = [
            ("F1", 59),
            ("F2", 60),
            ("F3", 61),
            ("F4", 62),
            ("F5", 63),
            ("F6", 64),
            ("F7", 65),
            ("F8", 66),
            ("F9", 67),
            ("F10", 68),
            ("F11", 87),
            ("F12", 88),
        ];
        for (label, code) in expected {
            assert_eq!(key_to_code(label), Some(code), "{label}");
            assert_eq!(
                key_to_code(&label.to_lowercase()),
                Some(code),
                "{label} lower"
            );
        }
        assert!(key_to_code("F13").is_none());
    }

    #[test]
    fn maps_number_and_letter_keycodes() {
        let expected = [
            ("0", 11),
            ("Q", 16),
            ("P", 25),
            ("A", 30),
            ("L", 38),
            ("Z", 44),
            ("M", 50),
        ];
        for (label, code) in expected {
            assert_eq!(key_to_code(label), Some(code), "{label}");
            assert_eq!(
                key_to_code(&label.to_lowercase()),
                Some(code),
                "{label} lower"
            );
        }
    }
}
