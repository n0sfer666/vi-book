use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("lua error: {0}")]
    Lua(#[from] mlua::Error),
}

impl Error {
    pub fn is_reported(&self) -> bool {
        let Self::Lua(error) = self;
        error.chain().any(|cause| {
            matches!(
                cause.downcast_ref::<v_ide_editor::execution::Error>(),
                Some(v_ide_editor::execution::Error::Autocmd(_))
            )
        })
    }
}
