use std::{
    fs::OpenOptions,
    io::{ErrorKind, Write},
    path::PathBuf,
};

use super::Document;
use crate::{
    disk::Stamp,
    io::error::Error,
    options::{Name, Value},
};

impl Document {
    pub(crate) fn rename_target(&mut self, name: String, path: PathBuf) {
        if self.path.as_ref() != Some(&path) {
            self.new_name = true;
            self.stamp = None;
            self.mark = None;
        }
        self.name = name;
        self.path = Some(path);
    }

    pub(crate) fn reset_new_file(&mut self) {
        self.take("");
        self.new_name = true;
        self.stamp = None;
        self.options.set(Name::Readonly, Value::Bool(false));
    }

    pub(super) fn create_new_target(&mut self) -> Result<(), Error> {
        let path = self.path.as_ref().ok_or(Error::NoPath)?;
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| {
                if error.kind() == ErrorKind::AlreadyExists {
                    Error::Exists
                } else {
                    Error::Io(error)
                }
            })?;
        file.write_all(self.editor.text().as_bytes())?;
        self.stamp = Stamp::of(path).ok();
        self.mark = None;
        self.new_name = false;
        self.mark_saved();
        Ok(())
    }
}
