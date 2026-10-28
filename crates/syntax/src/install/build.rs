use std::env;
use std::env::consts::DLL_EXTENSION;
use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use super::InstallError;
use super::command::{require, run};
use super::paths::grammar_root;
use super::query::highlights;
use super::report::each;
use crate::{LanguageSpec, Languages};

pub fn build(
    languages: &Languages,
    data: &Path,
    names: &[String],
    out: &mut dyn Write,
) -> Result<bool, InstallError> {
    let cc = compiler();
    require(OsStr::new(&cc[0]))?;
    each(languages, names, out, |spec| {
        compile(languages, data, spec, &cc)
    })
}

fn compiler() -> Vec<String> {
    let words: Vec<String> = env::var("CC")
        .unwrap_or_default()
        .split_whitespace()
        .map(str::to_string)
        .collect();
    match words.is_empty() {
        true => vec!["cc".to_string()],
        false => words,
    }
}

fn compile(
    languages: &Languages,
    data: &Path,
    spec: &LanguageSpec,
    cc: &[String],
) -> Result<(), String> {
    let src = grammar_root(data, spec).join("src");
    let parser = src.join("parser.c");
    if !parser.is_file() {
        return Err(format!("not fetched: {}", parser.display()));
    }
    let query = highlights(languages, data, spec)?;
    let library = data
        .join("grammars")
        .join(format!("{}.{DLL_EXTENSION}", spec.name));
    let mut command = Command::new(&cc[0]);
    command
        .args(&cc[1..])
        .args(["-shared", "-fPIC", "-O2", "-I"])
        .arg(&src)
        .arg(&parser);
    let scanner = src.join("scanner.c");
    if scanner.is_file() {
        command.arg(scanner);
    }
    write_parent(&library)?;
    run(command.arg("-o").arg(&library))?;
    let target = data.join("queries").join(&spec.name).join("highlights.scm");
    write_parent(&target)?;
    fs::write(&target, query).map_err(|err| err.to_string())
}

fn write_parent(path: &Path) -> Result<(), String> {
    match path.parent() {
        Some(parent) => fs::create_dir_all(parent).map_err(|err| err.to_string()),
        None => Ok(()),
    }
}
