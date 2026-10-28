mod common;

use std::fs;

use common::repo::toy_languages;
use common::{install_toy, scratch};
use v_ide_syntax::{Languages, canonical, generation, grammar, set_dir, set_languages};

#[test]
fn registry_resolves_fences_through_languages_and_caches() {
    let root = scratch("registry-toy");
    install_toy(&root, "toy");
    install_toy(&root, "stray");
    set_dir(root.clone());
    let before = generation();
    set_languages(Languages::parse(&toy_languages("file:///unused", "abc")).unwrap());
    assert!(generation() > before);
    assert_eq!(canonical("ty").as_deref(), Some("toy"));
    assert_eq!(canonical("TY"), None);
    assert!(grammar("ty").is_some());
    assert!(grammar("toy").is_some());
    assert!(grammar("stray").is_none(), "dylib without toml entry");
    assert!(grammar("nope").is_none());
    fs::remove_dir_all(root.join("grammars")).unwrap();
    assert!(grammar("toy").is_some(), "cached by canonical name");
    set_dir(scratch("registry-empty"));
    assert!(grammar("ty").is_none());
}
