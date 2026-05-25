use std::path::Path;

pub const SYSTEM_WINE_CANDIDATES: &[(&str, &str)] = &[
    ("/usr/bin/wine-cachyos", "Wine CachyOS"),
    ("/usr/bin/wine", "Wine"),
    ("/usr/bin/wine64", "Wine64"),
];

pub const WINETRICKS_BIN: &str = "/usr/bin/winetricks";

pub fn default_system_wine() -> &'static str {
    if Path::new(SYSTEM_WINE_CANDIDATES[0].0).exists() {
        SYSTEM_WINE_CANDIDATES[0].0
    } else {
        SYSTEM_WINE_CANDIDATES[1].0
    }
}

pub fn system_wine_available() -> bool {
    SYSTEM_WINE_CANDIDATES
        .iter()
        .take(2)
        .any(|(path, _)| Path::new(path).exists())
}

pub fn winetricks_available() -> bool {
    Path::new(WINETRICKS_BIN).exists()
}
