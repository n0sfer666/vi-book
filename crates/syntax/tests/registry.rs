mod common;

use std::fs;

use common::{install_toy, scratch};
use v_ide_syntax::{grammar, set_dir};

#[test]
fn registry_loads_from_set_dir_caches_and_resets() {
    let root = scratch("registry-toy");
    install_toy(&root, "toy");
    set_dir(root.clone());
    assert!(grammar("toy").is_some());
    assert!(grammar("nope").is_none());
    fs::remove_dir_all(root.join("grammars")).unwrap();
    assert!(grammar("toy").is_some());
    set_dir(scratch("registry-empty"));
    assert!(grammar("toy").is_none());
}
