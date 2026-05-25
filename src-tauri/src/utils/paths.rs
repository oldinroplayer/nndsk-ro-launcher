use std::path::PathBuf;

pub fn app_data_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(format!("{home}/.local/share/ro-launcher"))
}

pub fn data_file(name: &str) -> PathBuf {
    app_data_dir().join(name)
}
