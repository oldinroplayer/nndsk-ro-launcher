use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RunnerInfo {
    pub id: String,
    pub name: String,
    pub path: String,
}
