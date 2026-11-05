mod ex_io;

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
    pub(crate) new_name: bool,
    saved_text: String,
    read_tick: u64,
    stamp: Option<Stamp>,
    mark: Option<DiskMark>,
}

pub enum DiskCheck {
    Done(Option<Reaction>),
    Reload(DiskReload),
}

pub struct DiskReload {
    stamp: Stamp,
    pub(crate) keep_readonly: bool,
    text: String,
}

impl Document {
    pub fn new(text: &str) -> Self {
        let editor = Editor::new(text);
        Self {
            options: Values::default(),
            saved_text: editor.text(),
            read_tick: 0,
            editor,
            path: None,
            name: String::new(),
            new_name: false,
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
        doc.detect_readonly(false);
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
        if self.new_name {
            self.create_new_target()
        } else {
            self.write()
        }
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
        self.new_name = false;
        self.mark_saved();
        Ok(())
    }

    pub fn reload(&mut self) -> Result<(), Error> {
        self.reload_with_suffix("")?;
        self.detect_readonly(false);
        Ok(())
    }

    pub(crate) fn reload_with_suffix(&mut self, suffix: &str) -> Result<(), Error> {
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
        self.take(&append_suffix(text, suffix));
        Ok(())
    }

    pub fn check_disk(&mut self) -> Option<Reaction> {
        match self.prepare_disk_check() {
            DiskCheck::Done(reaction) => reaction,
            DiskCheck::Reload(reload) => {
                self.detect_readonly(reload.keep_readonly);
                self.apply_disk_reload(reload, "");
                Some(Reaction::Reload)
            }
        }
    }

    pub fn prepare_disk_check(&mut self) -> DiskCheck {
        let Some(path) = self.path.as_ref() else {
            return DiskCheck::Done(None);
        };
        let stamp = match Stamp::of(path) {
            Ok(stamp) => stamp,
            Err(err) if err.kind() == ErrorKind::NotFound => return DiskCheck::Done(self.vanish()),
            Err(_) => return DiskCheck::Done(None),
        };
        let keep_readonly = self.stamp.is_some_and(|old| old.same_mode(stamp));
        let dirty = self.is_dirty();
        let stale = self.mark == Some(DiskMark::Changed) && !dirty;
        if (self.stamp == Some(stamp) && !stale) || (!dirty && !self.editor.idle_normal()) {
            return DiskCheck::Done(None);
        }
        let text = match file::read(path) {
            Ok(text) => text,
            Err(Error::Io(err)) if err.kind() == ErrorKind::NotFound => {
                return DiskCheck::Done(self.vanish());
            }
            Err(_) => {
                self.stamp = Some(stamp);
                return DiskCheck::Done(self.conflict());
            }
        };
        if text == self.saved_text {
            self.stamp = Some(stamp);
            if !dirty {
                self.detect_readonly(keep_readonly);
            }
            self.mark = None;
            DiskCheck::Done(None)
        } else if dirty {
            self.stamp = Some(stamp);
            DiskCheck::Done(self.conflict())
        } else {
            DiskCheck::Reload(DiskReload {
                stamp,
                keep_readonly,
                text,
            })
        }
    }

    pub(crate) fn finish_disk_reload(
        &mut self,
        reload: DiskReload,
        suffix: &str,
    ) -> Result<(), Error> {
        let path = self.path.as_ref().ok_or(Error::NoPath)?;
        if Stamp::of(path).ok() != Some(reload.stamp) {
            return self.reload_with_suffix(suffix);
        }
        self.apply_disk_reload(reload, suffix);
        Ok(())
    }

    pub(crate) fn skip_disk_reload(&mut self, reload: DiskReload) {
        self.stamp = Some(reload.stamp);
        self.mark = None;
    }

    fn apply_disk_reload(&mut self, reload: DiskReload, suffix: &str) {
        self.stamp = Some(reload.stamp);
        self.take(&append_suffix(reload.text, suffix));
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

    pub fn read_tick(&self) -> u64 {
        self.read_tick
    }

    pub(crate) fn mark_read(&mut self) {
        self.mark_saved();
        self.read_tick = self.editor.change_tick();
    }

    pub fn filetype(&self) -> Option<&'static str> {
        match &self.path {
            Some(path) => filetype::detect(path),
            None => filetype::detect(Path::new(&self.name)),
        }
    }

    pub(crate) fn suspend_text_for_read(&mut self) -> (Editor, String, u64) {
        (
            std::mem::replace(&mut self.editor, Editor::new("")),
            std::mem::take(&mut self.saved_text),
            std::mem::take(&mut self.read_tick),
        )
    }

    pub(crate) fn restore_text_after_read(
        &mut self,
        (editor, saved_text, read_tick): (Editor, String, u64),
    ) {
        self.editor = editor;
        self.saved_text = saved_text;
        self.read_tick = read_tick;
    }

    pub(crate) fn detect_readonly(&mut self, keep: bool) {
        let detected = self.path.as_deref().is_some_and(file::readonly);
        let value = detected || (keep && self.readonly());
        self.options.set(Name::Readonly, Value::Bool(value));
    }

    fn take(&mut self, text: &str) {
        self.new_name = false;
        self.editor.replace_all(text);
        self.mark = None;
        self.mark_read();
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

fn append_suffix(mut text: String, suffix: &str) -> String {
    if !suffix.is_empty() {
        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }
        text.push_str(suffix);
    }
    text
}
