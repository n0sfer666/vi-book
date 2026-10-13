use std::ffi::OsString;
use std::path::{Path, PathBuf};

use v_ide::ui::config_path::resolve;

const NEW: &str = "/h/.config/v-ide/init.lua";
const OLD: &str = "/h/.config/vi-book/init.lua";
const ENV_WARNING: &str = "config: VIBOOK_CONFIG is deprecated; use V_IDE_CONFIG";
const PATH_WARNING: &str =
    "config: ~/.config/vi-book/init.lua is deprecated; move it to ~/.config/v-ide/init.lua";

fn env_of(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<OsString> {
    let owned: Vec<(String, String)> = pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    move |key| {
        owned
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| OsString::from(v))
    }
}

fn exists_of(paths: &[&str]) -> impl Fn(&Path) -> bool {
    let owned: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    move |path| owned.iter().any(|p| p == path)
}

#[test]
fn v_ide_config_wins_without_warning() {
    let env = env_of(&[
        ("V_IDE_CONFIG", "/a.lua"),
        ("VIBOOK_CONFIG", "/b.lua"),
        ("HOME", "/h"),
    ]);
    let got = resolve(env, exists_of(&[NEW, OLD]));
    assert_eq!(got, Some((PathBuf::from("/a.lua"), None)));
}

#[test]
fn vibook_config_is_fallback_with_warning() {
    let env = env_of(&[("VIBOOK_CONFIG", "/b.lua"), ("HOME", "/h")]);
    let got = resolve(env, exists_of(&[NEW, OLD]));
    assert_eq!(got, Some((PathBuf::from("/b.lua"), Some(ENV_WARNING))));
}

#[test]
fn new_path_wins_over_old_path() {
    let env = env_of(&[("HOME", "/h")]);
    let got = resolve(env, exists_of(&[NEW, OLD]));
    assert_eq!(got, Some((PathBuf::from(NEW), None)));
}

#[test]
fn old_path_is_fallback_with_warning() {
    let env = env_of(&[("HOME", "/h")]);
    let got = resolve(env, exists_of(&[OLD]));
    assert_eq!(got, Some((PathBuf::from(OLD), Some(PATH_WARNING))));
}

#[test]
fn no_config_when_no_file_exists() {
    let env = env_of(&[("HOME", "/h")]);
    assert_eq!(resolve(env, exists_of(&[])), None);
}

#[test]
fn no_config_without_home() {
    assert_eq!(resolve(env_of(&[]), exists_of(&[NEW, OLD])), None);
}

#[test]
fn empty_v_ide_config_is_unset() {
    let env = env_of(&[("V_IDE_CONFIG", ""), ("HOME", "/h")]);
    let got = resolve(env, exists_of(&[NEW]));
    assert_eq!(got, Some((PathBuf::from(NEW), None)));
}

#[test]
fn empty_vibook_config_is_unset() {
    let env = env_of(&[("VIBOOK_CONFIG", ""), ("HOME", "/h")]);
    let got = resolve(env, exists_of(&[OLD]));
    assert_eq!(got, Some((PathBuf::from(OLD), Some(PATH_WARNING))));
}

#[test]
fn relative_or_empty_home_is_ignored() {
    let relative = env_of(&[("HOME", "h")]);
    assert_eq!(
        resolve(relative, exists_of(&["h/.config/v-ide/init.lua"])),
        None
    );
    let empty = env_of(&[("HOME", "")]);
    assert_eq!(resolve(empty, exists_of(&[".config/v-ide/init.lua"])), None);
}

#[test]
fn userprofile_is_home_fallback() {
    let env = env_of(&[("HOME", ""), ("USERPROFILE", "/u")]);
    let got = resolve(env, exists_of(&["/u/.config/v-ide/init.lua"]));
    assert_eq!(
        got,
        Some((PathBuf::from("/u/.config/v-ide/init.lua"), None))
    );
}

#[test]
fn home_wins_over_userprofile() {
    let env = env_of(&[("HOME", "/h"), ("USERPROFILE", "/u")]);
    let got = resolve(env, exists_of(&[NEW, "/u/.config/v-ide/init.lua"]));
    assert_eq!(got, Some((PathBuf::from(NEW), None)));
}
