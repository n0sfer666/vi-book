use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    NoteOpen(PathBuf),
    NoteSave(PathBuf),
    NoteCreate(PathBuf),
}

impl Event {
    pub fn name(&self) -> &'static str {
        match self {
            Self::NoteOpen(_) => "note_open",
            Self::NoteSave(_) => "note_save",
            Self::NoteCreate(_) => "note_create",
        }
    }
}
