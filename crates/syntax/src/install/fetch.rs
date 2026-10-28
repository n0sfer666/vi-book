use std::ffi::OsStr;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use super::InstallError;
use super::command::{require, run};
use super::paths::source;
use super::report::each;
use crate::Languages;

pub fn fetch(
    languages: &Languages,
    data: &Path,
    names: &[String],
    out: &mut dyn Write,
) -> Result<bool, InstallError> {
    require(OsStr::new("git"))?;
    each(languages, names, out, |spec| {
        let dest = source(data, spec);
        fs::create_dir_all(&dest).map_err(|err| err.to_string())?;
        git(&dest, &["init", "--quiet"])?;
        let grammar = &spec.grammar;
        git(
            &dest,
            &[
                "fetch",
                "--quiet",
                "--depth",
                "1",
                "--end-of-options",
                &grammar.git,
                &grammar.rev,
            ],
        )?;
        git(&dest, &["checkout", "--quiet", "--force", "FETCH_HEAD"])
    })
}

fn git(dir: &Path, args: &[&str]) -> Result<(), String> {
    run(Command::new("git")
        .current_dir(dir)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .args(["-c", "advice.detachedHead=false"])
        .args(args))
}
