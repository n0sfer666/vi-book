use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("lua error: {0}")]
    Lua(#[from] mlua::Error),
}

#[derive(Debug, ThisError)]
#[error("{error}")]
pub(crate) struct Reported {
    pub checkpoint: v_ide_editor::execution::DiagnosticCheckpoint,
    #[source]
    pub error: v_ide_editor::execution::Error,
}

impl Error {
    pub fn is_reported(&self) -> bool {
        let Self::Lua(error) = self;
        error.chain().any(|cause| {
            cause.is::<Reported>()
                || matches!(
                    cause.downcast_ref::<v_ide_editor::execution::Error>(),
                    Some(v_ide_editor::execution::Error::Autocmd(_))
                )
        })
    }
    pub fn is_reported_since(
        &self,
        checkpoint: v_ide_editor::execution::DiagnosticCheckpoint,
    ) -> bool {
        let Self::Lua(error) = self;
        error.chain().any(|cause| {
            cause
                .downcast_ref::<Reported>()
                .is_some_and(|reported| reported.checkpoint.is_after(checkpoint))
        })
    }
}
