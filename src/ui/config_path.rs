use std::ffi::OsString;
use std::path::{Path, PathBuf};

const ENV_WARNING: &str = "config: VIBOOK_CONFIG is deprecated; use V_IDE_CONFIG";
const PATH_WARNING: &str =
    "config: ~/.config/vi-book/init.lua is deprecated; move it to ~/.config/v-ide/init.lua";

pub fn resolve(
    env: impl Fn(&str) -> Option<OsString>,
    exists: impl Fn(&Path) -> bool,
) -> Option<(PathBuf, Option<&'static str>)> {
    let set = |key: &str| env(key).filter(|value| !value.is_empty());
    if let Some(path) = set("V_IDE_CONFIG") {
        return Some((PathBuf::from(path), None));
    }
    if let Some(path) = set("VIBOOK_CONFIG") {
        return Some((PathBuf::from(path), Some(ENV_WARNING)));
    }
    let config = ["HOME", "USERPROFILE"]
        .into_iter()
        .filter_map(&set)
        .map(PathBuf::from)
        .find(|home| home.is_absolute())?
        .join(".config");
    let new = config.join("v-ide/init.lua");
    if exists(&new) {
        return Some((new, None));
    }
    let old = config.join("vi-book/init.lua");
    exists(&old).then_some((old, Some(PATH_WARNING)))
}
