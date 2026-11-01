use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapError {
    InvalidArgument,
    Unsupported(String),
    MissingCr,
    NoSuchMapping,
    AlreadyExists(String),
}

impl fmt::Display for MapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument => f.write_str("E474: Invalid argument"),
            Self::Unsupported(flag) => write!(f, "E474: Invalid argument ({flag} not supported)"),
            Self::MissingCr => f.write_str("E1255: <Cmd> mapping must end with <CR>"),
            Self::NoSuchMapping => f.write_str("E31: No such mapping"),
            Self::AlreadyExists(lhs) => write!(f, "E227: Mapping already exists for {lhs}"),
        }
    }
}

impl std::error::Error for MapError {}
