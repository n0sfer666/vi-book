use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum LoadError {
    BadName(String),
    NotFound(PathBuf),
    Open(String),
    Symbol(String),
    Abi(usize),
    Query(String),
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadName(name) => write!(f, "invalid language name: {name:?}"),
            Self::NotFound(path) => write!(f, "not found: {}", path.display()),
            Self::Open(reason) => write!(f, "cannot open library: {reason}"),
            Self::Symbol(symbol) => write!(f, "missing symbol: {symbol}"),
            Self::Abi(version) => write!(
                f,
                "incompatible ABI {version}, supported {}..={}",
                tree_sitter::MIN_COMPATIBLE_LANGUAGE_VERSION,
                tree_sitter::LANGUAGE_VERSION
            ),
            Self::Query(reason) => write!(f, "invalid highlights query: {reason}"),
        }
    }
}

impl std::error::Error for LoadError {}
