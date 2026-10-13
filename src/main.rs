use std::process::ExitCode;

fn main() -> ExitCode {
    match v_ide_tui::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("v-ide: {err}");
            ExitCode::FAILURE
        }
    }
}
