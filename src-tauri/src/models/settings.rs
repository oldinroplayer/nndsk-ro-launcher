use serde::{Deserialize, Serialize};

use crate::utils::default_system_wine;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub default_runner: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            default_runner: default_system_wine().to_string(),
        }
    }
}
