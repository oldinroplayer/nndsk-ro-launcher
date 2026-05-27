use serde::{Deserialize, Serialize};

fn default_spammer_delay_ms() -> u64 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SpammerConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_spammer_delay_ms")]
    pub delay_ms: u64,
}

impl Default for SpammerConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            delay_ms: default_spammer_delay_ms(),
        }
    }
}

impl SpammerConfig {
    pub fn clamped(&self) -> Self {
        let mut c = self.clone();
        c.delay_ms = c.delay_ms.clamp(5, 100);
        c
    }
}
