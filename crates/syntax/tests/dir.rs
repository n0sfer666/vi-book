use std::ffi::OsString;
use std::path::PathBuf;

use v_ide_syntax::{config_dir, data_dir};

fn os(text: &str) -> Option<OsString> {
    Some(OsString::from(text))
}

#[test]
fn xdg_data_home_wins_over_home() {
    assert_eq!(
        data_dir(os("/xdg"), os("/home/u")),
        Some(PathBuf::from("/xdg/v-ide"))
    );
}

#[test]
fn home_falls_back_to_local_share() {
    assert_eq!(
        data_dir(None, os("/home/u")),
        Some(PathBuf::from("/home/u/.local/share/v-ide"))
    );
}

#[test]
fn relative_or_empty_paths_are_ignored() {
    assert_eq!(
        data_dir(os("xdg"), os("/home/u")),
        Some(PathBuf::from("/home/u/.local/share/v-ide"))
    );
    assert_eq!(data_dir(os(""), os("home")), None);
    assert_eq!(data_dir(None, os("")), None);
    assert_eq!(data_dir(None, None), None);
}

#[test]
fn xdg_config_home_wins_over_home() {
    assert_eq!(
        config_dir(os("/xdg"), os("/home/u")),
        Some(PathBuf::from("/xdg/v-ide"))
    );
}

#[test]
fn config_falls_back_to_dot_config() {
    assert_eq!(
        config_dir(os("xdg"), os("/home/u")),
        Some(PathBuf::from("/home/u/.config/v-ide"))
    );
    assert_eq!(config_dir(None, os("home")), None);
}
