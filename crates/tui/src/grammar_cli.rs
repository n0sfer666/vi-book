use std::io;
use std::path::Path;
use std::process::ExitCode;

use v_ide_syntax::install::{self, InstallError};
use v_ide_syntax::{Languages, default_config_dir, default_dir};

const USAGE: &str = "usage: v-ide grammar fetch|build|check [LANG...]";

type Step = fn(&Languages, &Path, &[String], &mut dyn io::Write) -> Result<bool, InstallError>;

pub fn run(args: &[String]) -> Option<ExitCode> {
    let (first, rest) = args.split_first()?;
    if first != "grammar" {
        return None;
    }
    Some(match execute(rest) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::FAILURE,
        Err(message) => {
            eprintln!("v-ide: {message}");
            ExitCode::FAILURE
        }
    })
}

fn execute(args: &[String]) -> Result<bool, String> {
    let (sub, names) = args.split_first().ok_or(USAGE)?;
    let step: Step = match sub.as_str() {
        "fetch" => install::fetch,
        "build" => install::build,
        "check" => install::check,
        _ => return Err(USAGE.to_string()),
    };
    let languages = match default_config_dir() {
        Some(dir) => Languages::load(&dir).map_err(|err| err.to_string())?,
        None => Languages::builtin(),
    };
    let data = default_dir().ok_or("no data directory: set XDG_DATA_HOME or HOME")?;
    step(&languages, &data, names, &mut io::stdout().lock()).map_err(|err| err.to_string())
}
