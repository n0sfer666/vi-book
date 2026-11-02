use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("buffer has no associated file path")]
    NoPath,
    #[error("E45: 'readonly' option is set (add ! to override)")]
    Readonly,
}
