use crate::models::server::ServerConfig;
use crate::utils::{read_json, servers_path, write_json};

#[tauri::command]
pub async fn list_servers() -> Result<Vec<ServerConfig>, String> {
    read_json(&servers_path())
}

#[tauri::command]
pub async fn save_servers(servers: Vec<ServerConfig>) -> Result<(), String> {
    write_json(&servers_path(), &servers)
}
