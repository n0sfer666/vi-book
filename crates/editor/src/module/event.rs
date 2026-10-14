use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    NoteOpen(PathBuf),
    NoteSave(PathBuf),
    NoteCreate(PathBuf),
    VaultChanged {
        notes: Vec<String>,
        tags: Vec<String>,
        backlinks: HashMap<String, Vec<String>>,
    },
    FollowLink(String),
}

impl Event {
    pub fn name(&self) -> &'static str {
        match self {
            Self::NoteOpen(_) => "note_open",
            Self::NoteSave(_) => "note_save",
            Self::NoteCreate(_) => "note_create",
            Self::VaultChanged { .. } => "vault_changed",
            Self::FollowLink(_) => "follow_link",
        }
    }
}
