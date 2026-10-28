#![forbid(unsafe_code)]

use std::process::ExitCode;

fn main() -> ExitCode {
    #[cfg(feature = "highlight")]
    {
        let args: Vec<String> = std::env::args_os()
            .skip(1)
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        if let Some(code) = v_ide_tui::grammar_cli::run(&args) {
            return code;
        }
    }
    match v_ide_tui::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("v-ide: {err}");
            ExitCode::FAILURE
        }
    }
}
