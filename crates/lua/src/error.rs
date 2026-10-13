use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("lua error: {0}")]
    Lua(#[from] mlua::Error),
}
