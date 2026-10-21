use std::fs;
use std::path::{Path, PathBuf};

pub fn scratch_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(name)
}

pub fn scratch(label: &str) -> PathBuf {
    let root = scratch_path(label);
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}
