use std::sync::{Arc, Mutex};

use ro_tools_core::{HeldKeyWriter, KeyPressWriter, PointerWriter, ToolsError};
use ro_tools_linux::LazyYdotoolInput;

/// Cola global de input: serializa ydotool entre AutoPot, Spammer y futuro AutoBuff.
#[derive(Clone)]
pub struct InputGateway {
    inner: Arc<Mutex<LazyYdotoolInput>>,
}

impl InputGateway {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(LazyYdotoolInput::new())),
        }
    }

    pub fn reset(&self) {
        if let Ok(guard) = self.inner.lock() {
            guard.reset();
        }
    }

    pub fn writer(&self) -> GatewayWriter {
        GatewayWriter(Arc::clone(&self.inner))
    }
}

#[derive(Clone)]
pub struct GatewayWriter(Arc<Mutex<LazyYdotoolInput>>);

impl KeyPressWriter for GatewayWriter {
    fn press_key(&self, key: &str) -> Result<(), ToolsError> {
        let guard = self
            .0
            .lock()
            .map_err(|_| ToolsError::Other("input gateway lock poisoned".into()))?;
        guard.press_key(key)
    }
}

impl PointerWriter for GatewayWriter {
    fn click_left(&self) -> Result<(), ToolsError> {
        let guard = self
            .0
            .lock()
            .map_err(|_| ToolsError::Other("input gateway lock poisoned".into()))?;
        guard.click_left()
    }
}

impl HeldKeyWriter for GatewayWriter {
    fn key_down(&self, key: &str) -> Result<(), ToolsError> {
        let guard = self
            .0
            .lock()
            .map_err(|_| ToolsError::Other("input gateway lock poisoned".into()))?;
        guard.key_down(key)
    }

    fn key_up(&self, key: &str) -> Result<(), ToolsError> {
        let guard = self
            .0
            .lock()
            .map_err(|_| ToolsError::Other("input gateway lock poisoned".into()))?;
        guard.key_up(key)
    }
}
