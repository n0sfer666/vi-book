use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum LanguagesError {
    Parse(String),
    File(PathBuf, String),
}

impl fmt::Display for LanguagesError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Parse(reason) => write!(f, "invalid languages.toml: {reason}"),
            Self::File(path, reason) => write!(f, "{}: {reason}", path.display()),
        }
    }
}

impl std::error::Error for LanguagesError {}
