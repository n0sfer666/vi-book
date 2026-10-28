#![allow(dead_code)]

use std::env::consts::DLL_EXTENSION;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod repo;

const PARSER_C: &str = include_str!("../fixtures/toy/src/parser.c");
const PARSER_H: &str = include_str!("../fixtures/toy/src/tree_sitter/parser.h");
pub const HIGHLIGHTS: &str = include_str!("../fixtures/toy/highlights.scm");

pub fn scratch(label: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(label);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

pub fn library(root: &Path, lang: &str) -> PathBuf {
    root.join("grammars")
        .join(format!("{lang}.{DLL_EXTENSION}"))
}

pub fn query(root: &Path, lang: &str) -> PathBuf {
    root.join("queries").join(lang).join("highlights.scm")
}

pub fn write_query(root: &Path, lang: &str, text: &str) {
    let path = query(root, lang);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

pub fn compile(root: &Path, lang: &str, source: &str, defines: &[String]) {
    let src = root.join("src").join(lang);
    fs::create_dir_all(src.join("tree_sitter")).unwrap();
    fs::write(src.join("tree_sitter").join("parser.h"), PARSER_H).unwrap();
    fs::write(src.join("parser.c"), source).unwrap();
    let out = library(root, lang);
    fs::create_dir_all(out.parent().unwrap()).unwrap();
    let status = Command::new("cc")
        .args(["-shared", "-fPIC", "-O0", "-I"])
        .arg(&src)
        .args(defines)
        .arg(src.join("parser.c"))
        .arg("-o")
        .arg(&out)
        .status()
        .unwrap();
    assert!(status.success(), "cc failed for {lang}");
}

pub fn install_toy(root: &Path, lang: &str) {
    let symbol = format!("-Dtree_sitter_toy=tree_sitter_{}", lang.replace('-', "_"));
    compile(root, lang, PARSER_C, &[symbol]);
    write_query(root, lang, HIGHLIGHTS);
}
