use std::fmt;
use std::io;

#[derive(Debug)]
pub enum InstallError {
    MissingTool(String),
    Output(io::Error),
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingTool(tool) => write!(f, "{tool} not found in PATH"),
            Self::Output(err) => write!(f, "cannot write output: {err}"),
        }
    }
}

impl std::error::Error for InstallError {}

impl From<io::Error> for InstallError {
    fn from(err: io::Error) -> Self {
        Self::Output(err)
    }
}
