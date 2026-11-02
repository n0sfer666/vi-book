use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use crate::disk::{DiskMark, Reaction, Stamp};
use crate::filetype;
use crate::io::error::Error;
use crate::io::file;
use crate::options::{Name, Value, Values};
use v_ide_engine::editor::Editor;

pub struct Document {
    pub editor: Editor,
    pub path: Option<PathBuf>,
    pub name: String,
    pub(crate) options: Values,
    saved_text: String,
    stamp: Option<Stamp>,
    mark: Option<DiskMark>,
}

impl Document {
    pub fn new(text: &str) -> Self {
        let editor = Editor::new(text);
        Self {
            options: Values::default(),
            saved_text: editor.text(),
            editor,
            path: None,
            name: String::new(),
            stamp: None,
            mark: None,
        }
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref();
        let stamp = Stamp::of(path).ok();
        let mut doc = Self::new(&file::read(path)?);
        doc.path = Some(path.to_path_buf());
        doc.stamp = stamp;
        Ok(doc)
    }

    pub fn readonly(&self) -> bool {
        self.options.get(Name::Readonly) == Value::Bool(true)
    }

    pub fn save(&mut self) -> Result<(), Error> {
        if self.path.is_none() {
            return Err(Error::NoPath);
        }
        if self.readonly() {
            return Err(Error::Readonly);
        }
        self.write()
    }

    pub fn save_force(&mut self) -> Result<(), Error> {
        if self.path.is_none() {
            return Err(Error::NoPath);
        }
        self.options.set(Name::Readonly, Value::Bool(false));
        self.write()
    }

    fn write(&mut self) -> Result<(), Error> {
        let path = self.path.as_ref().ok_or(Error::NoPath)?;
        file::write(path, &self.editor.text())?;
        self.stamp = Stamp::of(path).ok();
        self.mark = None;
        self.mark_saved();
        Ok(())
    }

    pub fn reload(&mut self) -> Result<(), Error> {
        let path = self.path.as_ref().ok_or(Error::NoPath)?;
        let stamp = Stamp::of(path).ok();
        let text = match file::read(path) {
            Ok(text) => text,
            Err(Error::Io(err)) if err.kind() == ErrorKind::NotFound => {
                self.vanish();
                return Err(Error::Io(err));
            }
            Err(err) => return Err(err),
        };
        self.stamp = stamp;
        self.take(&text);
        Ok(())
    }

    pub fn check_disk(&mut self) -> Option<Reaction> {
        let path = self.path.as_ref()?;
        let stamp = match Stamp::of(path) {
            Ok(stamp) => stamp,
            Err(err) if err.kind() == ErrorKind::NotFound => return self.vanish(),
            Err(_) => return None,
        };
        let stale = self.mark == Some(DiskMark::Changed) && !self.is_dirty();
        if self.stamp == Some(stamp) && !stale {
            return None;
        }
        if !self.is_dirty() && !self.editor.idle_normal() {
            return None;
        }
        let text = match file::read(path) {
            Ok(text) => text,
            Err(Error::Io(err)) if err.kind() == ErrorKind::NotFound => return self.vanish(),
            Err(_) => {
                self.stamp = Some(stamp);
                return self.conflict();
            }
        };
        self.stamp = Some(stamp);
        if text == self.saved_text {
            self.mark = None;
            None
        } else if self.is_dirty() {
            self.conflict()
        } else {
            self.take(&text);
            Some(Reaction::Reload)
        }
    }

    pub fn disk_mark(&self) -> Option<DiskMark> {
        self.mark
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

    fn take(&mut self, text: &str) {
        self.editor.replace_all(text);
        self.mark = None;
        self.mark_saved();
    }

    fn conflict(&mut self) -> Option<Reaction> {
        if !self.is_dirty() {
            return None;
        }
        self.mark = Some(DiskMark::Changed);
        Some(Reaction::Conflict)
    }

    fn vanish(&mut self) -> Option<Reaction> {
        self.stamp.take()?;
        self.mark = Some(DiskMark::Deleted);
        Some(Reaction::Vanished)
    }
}
