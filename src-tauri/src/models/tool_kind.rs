use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolKind {
    OpenSetup,
    Patcher,
    DgVoodoo,
}

impl ToolKind {
    pub fn needs_dgvoodoo_overrides(self) -> bool {
        matches!(self, ToolKind::OpenSetup | ToolKind::DgVoodoo)
    }
}
