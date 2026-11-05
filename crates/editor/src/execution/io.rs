use std::{
    io::{self, ErrorKind},
    path::PathBuf,
};

use super::{Context, Error};
use crate::{
    Error as FileError,
    disk::Reaction,
    document::{DiskReload, Document},
    options::{Access, Name},
    workspace::DocId,
};

impl Context<'_> {
    pub fn open_existing_path(&self, name: &str, startup: bool) -> Result<(), Error> {
        let (path, current, reuse) = {
            let workspace = self.workspace.borrow();
            let current = workspace.current_id();
            (
                workspace.absolute(name),
                current,
                startup && workspace.current().path.is_none() && workspace.discardable(current),
            )
        };
        let metadata = path.metadata().map_err(FileError::Io)?;
        if metadata.is_dir() {
            return Err(FileError::Io(io::Error::from(ErrorKind::IsADirectory)).into());
        }
        if reuse {
            self.workspace
                .borrow_mut()
                .current_mut()
                .rename_target(name.to_owned(), path);
            return self.read_document(current, true, true);
        }
        self.open_document(name.to_owned(), path, false, startup)
    }

    pub fn reload_current(&self) -> Result<(), Error> {
        let id = self.workspace.borrow().current_id();
        self.read_document(id, false, false)
    }

    pub fn reload_disk(&self, id: DocId, reload: DiskReload) -> Result<Option<Reaction>, Error> {
        let (path, cursor) = {
            let mut workspace = self.workspace.borrow_mut();
            let doc = workspace.doc_mut(id).ok_or(FileError::NoPath)?;
            (doc.path.clone(), doc.editor.cursor_state())
        };
        let suffix = match self.read_pre(id, true, reload.keep_readonly) {
            Ok(Some(suffix)) => suffix,
            cancelled => {
                if let Some(doc) = self.workspace.borrow_mut().doc_mut(id)
                    && doc.path == path
                {
                    doc.skip_disk_reload(reload);
                }
                return cancelled.map(|_| None);
            }
        };
        {
            let mut workspace = self.workspace.borrow_mut();
            let doc = workspace.current_mut();
            doc.finish_disk_reload(reload, &suffix)?;
            doc.editor.set_cursor(Default::default());
        }

        let result = self.read_post("BufReadPost", id).and_then(|()| {
            if self.buffer(0) == Some(id) {
                self.emit_filetype(id)?;
            }
            Ok(())
        });
        if let Some(doc) = self.workspace.borrow_mut().doc_mut(id) {
            doc.editor.restore_cursor_state(&cursor);
        }
        result?;
        if self.buffer(0) == Some(id) {
            self.process_modelines();
        }
        Ok(Some(Reaction::Reload))
    }

    pub(super) fn edit_buffer(&self, input: &str, force: bool) -> Result<(), Error> {
        let (name, path, existing, current, dirty) = {
            let workspace = self.workspace.borrow();
            let doc = workspace.current();
            let (name, path) = if input.is_empty() {
                (doc.name.clone(), doc.path.clone().ok_or(FileError::NoPath)?)
            } else {
                let name = super::filename::parse(input)?;
                let path = workspace.absolute(&name);
                (name, path)
            };
            let existing = workspace.find_path(&path);
            (name, path, existing, workspace.current_id(), doc.is_dirty())
        };
        if let Some(id) = existing {
            if id != current {
                return self.select_document(id);
            }
            if dirty && !force {
                return Err(Error::Ex(
                    "E37: No write since last change (add ! to override)".into(),
                ));
            }
            return match self.read_document(id, false, false) {
                Err(Error::File(FileError::Io(error))) if error.kind() == ErrorKind::NotFound => {
                    self.workspace.borrow_mut().current_mut().reset_new_file();
                    self.new_file(id)
                }
                result => result,
            };
        }
        let missing = match path.metadata() {
            Ok(metadata) if metadata.is_dir() => {
                return Err(FileError::Io(io::Error::from(ErrorKind::IsADirectory)).into());
            }
            Ok(_) => false,
            Err(error) if error.kind() == ErrorKind::NotFound => true,
            Err(error) => return Err(FileError::Io(error).into()),
        };
        self.open_document(name, path, missing, false)
    }

    fn open_document(
        &self,
        name: String,
        path: PathBuf,
        missing: bool,
        startup: bool,
    ) -> Result<(), Error> {
        let id = {
            let mut workspace = self.workspace.borrow_mut();
            let mut doc = Document::new("");
            doc.name = name;
            doc.path = Some(path);
            doc.new_name = missing;
            doc.options.set(
                Name::Modeline,
                workspace.option(Name::Modeline, Access::Global),
            );
            workspace.add(doc)
        };
        self.new_document(id)?;
        if !self.leave_for(id)? {
            return Ok(());
        }
        if missing {
            self.new_file(id)
        } else {
            self.read_document(id, true, startup)
        }
    }

    fn new_file(&self, id: DocId) -> Result<(), Error> {
        self.finish_read("BufNewFile", id, false)
    }

    fn read_document(&self, id: DocId, initial: bool, startup: bool) -> Result<(), Error> {
        let (path, cursor) = {
            let mut workspace = self.workspace.borrow_mut();
            let doc = workspace.doc_mut(id).ok_or(FileError::NoPath)?;
            (
                doc.path.clone().ok_or(FileError::NoPath)?,
                doc.editor.cursor_state(),
            )
        };
        if !initial && matches!(path.try_exists(), Ok(false)) {
            let mut workspace = self.workspace.borrow_mut();
            return Ok(workspace.current_mut().reload()?);
        }
        let suffix = self
            .read_pre(id, !initial, false)?
            .ok_or_else(|| Error::Ex("E201: Autocommands changed buffer or file".into()))?;
        {
            let mut workspace = self.workspace.borrow_mut();
            let doc = workspace.current_mut();
            doc.reload_with_suffix(&suffix)?;
            doc.editor.set_cursor(Default::default());
        }
        let result = self.finish_read("BufReadPost", id, startup);
        if !initial && let Some(doc) = self.workspace.borrow_mut().doc_mut(id) {
            doc.editor.restore_cursor_state(&cursor);
        }
        result
    }

    fn finish_read(&self, event: &str, id: DocId, startup: bool) -> Result<(), Error> {
        self.emit_filetype(id)?;
        self.read_post(event, id)?;
        if self.buffer(0) == Some(id) {
            self.enter_read_document(id, startup)?;
        }
        Ok(())
    }

    fn read_pre(
        &self,
        id: DocId,
        clear: bool,
        keep_readonly: bool,
    ) -> Result<Option<String>, Error> {
        let (path, editor) = {
            let mut workspace = self.workspace.borrow_mut();
            let doc = workspace.doc_mut(id).ok_or(FileError::NoPath)?;
            doc.detect_readonly(keep_readonly);
            let editor = clear.then(|| doc.suspend_text_for_read());
            (doc.path.clone(), editor)
        };
        let pre = self.emit_io("BufReadPre", id);
        let mut workspace = self.workspace.borrow_mut();
        let current = workspace.current_id() == id;
        let Some(doc) = workspace.doc_mut(id) else {
            return pre.map(|_| None);
        };
        let suffix = doc.editor.text();
        if let Some(editor) = editor {
            doc.restore_text_after_read(editor);
        }
        pre?;
        Ok((current && doc.path == path).then_some(suffix))
    }

    fn read_post(&self, event: &str, id: DocId) -> Result<(), Error> {
        if self.buffer(0) == Some(id) {
            return self.read_post_current(event, id);
        }
        let Some(scope) = self.workspace.borrow_mut().enter_event_scope(id) else {
            return Ok(());
        };
        let prior_navigation = self.navigated.replace(false);
        let result = self.read_post_current(event, id);
        let navigated = self.navigated.get();
        self.navigated.set(prior_navigation || navigated);
        self.workspace
            .borrow_mut()
            .finish_event_scope(scope, navigated);
        result
    }

    fn read_post_current(&self, event: &str, id: DocId) -> Result<(), Error> {
        let result = self.emit_io(event, id);
        let mut workspace = self.workspace.borrow_mut();
        if workspace.current_id() == id {
            workspace.current_mut().mark_saved();
        }
        result
    }
}
