use std::path::{Path, PathBuf};

use crate::io::error::Error;
use crate::io::file;
use v_ide_engine::editor::Editor;

pub struct Document {
    pub editor: Editor,
    pub path: Option<PathBuf>,
}

impl Document {
    pub fn new(text: &str) -> Self {
        Self {
            editor: Editor::new(text),
            path: None,
        }
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let text = file::read(path.as_ref())?;
        Ok(Self {
            editor: Editor::new(&text),
            path: Some(path.as_ref().to_path_buf()),
        })
    }

    pub fn save(&mut self) -> Result<(), Error> {
        let path = self.path.as_ref().ok_or(Error::NoPath)?;
        file::write(path, &self.editor.text())
    }
}
