use std::path::PathBuf;

use crate::utils::data_file;

pub fn servers_path() -> PathBuf {
    data_file("servers.json")
}
