use std::path::{Path, PathBuf};

use crate::filetype;
use crate::io::error::Error;
use crate::io::file;
use v_ide_engine::editor::Editor;

pub struct Document {
    pub editor: Editor,
    pub path: Option<PathBuf>,
    pub name: String,
    saved_text: String,
}

impl Document {
    pub fn new(text: &str) -> Self {
        let editor = Editor::new(text);
        Self {
            saved_text: editor.text(),
            editor,
            path: None,
            name: String::new(),
        }
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let mut doc = Self::new(&file::read(path.as_ref())?);
        doc.path = Some(path.as_ref().to_path_buf());
        Ok(doc)
    }

    pub fn save(&mut self) -> Result<(), Error> {
        let path = self.path.as_ref().ok_or(Error::NoPath)?;
        file::write(path, &self.editor.text())?;
        self.mark_saved();
        Ok(())
    }

    pub fn is_dirty(&self) -> bool {
        self.editor.text() != self.saved_text
    }

    pub fn mark_saved(&mut self) {
        self.saved_text = self.editor.text();
    }

    pub fn filetype(&self) -> Option<&'static str> {
        match &self.path {
            Some(path) => filetype::detect(path),
            None => filetype::detect(Path::new(&self.name)),
        }
    }
}
