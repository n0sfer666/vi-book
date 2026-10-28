use std::ffi::OsStr;
use std::process::{Command, Stdio};

use super::InstallError;

pub(crate) fn require(program: &OsStr) -> Result<(), InstallError> {
    let probe = Command::new(program)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let missing = match probe {
        Ok(status) => !status.success(),
        Err(_) => true,
    };
    match missing {
        true => Err(InstallError::MissingTool(
            program.to_string_lossy().into_owned(),
        )),
        false => Ok(()),
    }
}

pub(crate) fn run(command: &mut Command) -> Result<(), String> {
    let output = command.output().map_err(|err| err.to_string())?;
    if output.status.success() {
        return Ok(());
    }
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Err(if stderr.is_empty() {
        output.status.to_string()
    } else {
        stderr
    })
}
