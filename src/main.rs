use std::process::ExitCode;

fn main() -> ExitCode {
    match vi_book::ui::run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("vi-book: {err}");
            ExitCode::FAILURE
        }
    }
}
