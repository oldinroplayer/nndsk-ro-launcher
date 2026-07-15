use std::sync::{Arc, Mutex};
use std::time::Duration;

use ro_tools_core::{
    CombatInputBackend, HeldKeyWriter, KeyPressWriter, PointerWriter, SpamCycleWriter, ToolsError,
};
use ro_tools_linux::LazyYdotoolInput;

use super::uinput_worker::{InputSource, MetricsSnapshot, UinputInput, UinputWriter};

/// Shared input gateway. Combat input uses one persistent, prioritized uinput
/// worker; ydotool remains available for compatibility and AutoBuff.
#[derive(Clone)]
pub struct InputGateway {
    ydotool: Arc<Mutex<LazyYdotoolInput>>,
    uinput: UinputInput,
}

impl InputGateway {
    pub fn new() -> Self {
        Self {
            ydotool: Arc::new(Mutex::new(LazyYdotoolInput::new())),
            uinput: UinputInput::new(),
        }
    }

    pub fn reset_ydotool(&self) {
        if let Ok(guard) = self.ydotool.lock() {
            guard.reset();
        }
    }

    pub fn ydotool_writer(&self) -> GatewayWriter {
        GatewayWriter::Ydotool(Arc::clone(&self.ydotool))
    }

    pub fn writer_for(
        &self,
        backend: CombatInputBackend,
        source: InputSource,
        effective_delay_ms: u64,
    ) -> Result<GatewayWriter, ToolsError> {
        match backend {
            CombatInputBackend::Uinput => Ok(GatewayWriter::Uinput(
                self.uinput
                    .writer(source, Duration::from_millis(effective_delay_ms.max(10)))?,
            )),
            CombatInputBackend::Ydotool => Ok(self.ydotool_writer()),
        }
    }

    pub async fn prepare_uinput(&self) -> Result<String, ToolsError> {
        let uinput = self.uinput.clone();
        tokio::task::spawn_blocking(move || uinput.prepare())
            .await
            .map_err(|error| {
                ToolsError::Other(format!(
                    "uinput stage=join preparation device=both errno=none: {error}"
                ))
            })?
    }

    pub fn is_uinput_prepared(&self) -> bool {
        self.uinput.is_prepared()
    }

    pub fn uinput_metrics(&self, source: InputSource) -> MetricsSnapshot {
        self.uinput.snapshot_metrics(source)
    }

    pub fn shutdown(&self) {
        self.uinput.shutdown();
    }
}

impl Default for InputGateway {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone)]
pub enum GatewayWriter {
    Uinput(UinputWriter),
    Ydotool(Arc<Mutex<LazyYdotoolInput>>),
}

impl KeyPressWriter for GatewayWriter {
    fn press_key(&self, key: &str) -> Result<(), ToolsError> {
        match self {
            Self::Uinput(writer) => writer.press_key(key),
            Self::Ydotool(inner) => {
                let guard = inner
                    .lock()
                    .map_err(|_| ToolsError::Other("input gateway lock poisoned".into()))?;
                guard.press_key(key)
            }
        }
    }
}

impl PointerWriter for GatewayWriter {
    fn click_left(&self) -> Result<(), ToolsError> {
        match self {
            Self::Uinput(_) => Err(ToolsError::Other(
                "uinput click aislado no permitido; use SpamCycle".into(),
            )),
            Self::Ydotool(inner) => {
                let guard = inner
                    .lock()
                    .map_err(|_| ToolsError::Other("input gateway lock poisoned".into()))?;
                guard.click_left()
            }
        }
    }
}

impl HeldKeyWriter for GatewayWriter {
    fn key_down(&self, key: &str) -> Result<(), ToolsError> {
        match self {
            Self::Uinput(writer) => writer.key_event(key, 1),
            Self::Ydotool(inner) => {
                let guard = inner
                    .lock()
                    .map_err(|_| ToolsError::Other("input gateway lock poisoned".into()))?;
                guard.key_down(key)
            }
        }
    }

    fn key_up(&self, key: &str) -> Result<(), ToolsError> {
        match self {
            Self::Uinput(writer) => writer.key_event(key, 0),
            Self::Ydotool(inner) => {
                let guard = inner
                    .lock()
                    .map_err(|_| ToolsError::Other("input gateway lock poisoned".into()))?;
                guard.key_up(key)
            }
        }
    }
}

impl SpamCycleWriter for GatewayWriter {
    fn spam_cycle(
        &self,
        key: &str,
        deadline: Option<std::time::Instant>,
    ) -> Result<bool, ToolsError> {
        match self {
            Self::Uinput(writer) => writer.spam_cycle(key, deadline),
            Self::Ydotool(inner) => {
                // Hold the outer gateway lock for the whole compatibility sequence too.
                let guard = inner
                    .lock()
                    .map_err(|_| ToolsError::Other("input gateway lock poisoned".into()))?;
                guard.key_down(key)?;
                guard.click_left()?;
                guard.key_up(key)?;
                Ok(true)
            }
        }
    }
}
