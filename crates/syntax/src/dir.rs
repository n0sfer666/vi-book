use std::env;
use std::ffi::OsString;
use std::path::PathBuf;

pub fn default_dir() -> Option<PathBuf> {
    data_dir(env::var_os("XDG_DATA_HOME"), env::var_os("HOME"))
}

pub fn default_config_dir() -> Option<PathBuf> {
    config_dir(env::var_os("XDG_CONFIG_HOME"), env::var_os("HOME"))
}

pub fn data_dir(xdg: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    app_dir(xdg, home, ".local/share")
}

pub fn config_dir(xdg: Option<OsString>, home: Option<OsString>) -> Option<PathBuf> {
    app_dir(xdg, home, ".config")
}

fn app_dir(xdg: Option<OsString>, home: Option<OsString>, fallback: &str) -> Option<PathBuf> {
    let absolute = |value: OsString| Some(PathBuf::from(value)).filter(|path| path.is_absolute());
    let base = xdg
        .and_then(absolute)
        .or_else(|| home.and_then(absolute).map(|home| home.join(fallback)))?;
    Some(base.join("v-ide"))
}
