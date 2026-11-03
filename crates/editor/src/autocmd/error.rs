use super::{AutocmdId, GroupId};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("E936: Cannot delete the current group")]
    CurrentGroup,
    #[error(transparent)]
    Pattern(#[from] super::PatternError),
    #[error("Invalid event: {0}")]
    InvalidEvent(String),
    #[error("No such group: {0}")]
    GroupId(GroupId),
    #[error("No such group: {0}")]
    GroupName(String),
    #[error("Invalid autocmd id: {0}")]
    AutocmdId(AutocmdId),
    #[error("Invalid buffer number: {0}")]
    Buffer(u64),
}
