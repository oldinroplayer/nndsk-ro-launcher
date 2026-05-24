use crate::models::server::ServerConfig;
use crate::utils::{app_data_dir, read_json, write_json};

fn servers_path() -> std::path::PathBuf {
    app_data_dir().join("servers.json")
}

#[tauri::command]
pub async fn list_servers() -> Result<Vec<ServerConfig>, String> {
    read_json(&servers_path())
}

#[tauri::command]
pub async fn save_servers(servers: Vec<ServerConfig>) -> Result<(), String> {
    write_json(&servers_path(), &servers)
}
