use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    Unknown,
    Invalid,
    Positive,
    Number,
    Trailing,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub kind: ErrorKind,
    pub argument: String,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind, argument: &str) -> Self {
        Self {
            kind,
            argument: argument.to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = match self.kind {
            ErrorKind::Unknown => "E518: Unknown option",
            ErrorKind::Invalid => "E474: Invalid argument",
            ErrorKind::Positive => "E487: Argument must be positive",
            ErrorKind::Number => "E521: Number required after =",
            ErrorKind::Trailing => "E488: Trailing characters",
        };
        write!(f, "{prefix}: {}", self.argument)
    }
}

impl std::error::Error for Error {}
