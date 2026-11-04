use super::{Context, Error};
use crate::{Error as FileError, disk::DiskMark};

impl Context<'_> {
    pub fn write_current(&self, force: bool) -> Result<(), Error> {
        self.write_buffer("", force)
    }

    pub(super) fn write_buffer(&self, input: &str, force: bool) -> Result<(), Error> {
        let id = self.workspace.borrow().current_id();
        if !input.is_empty() {
            let workspace = self.workspace.borrow();
            let name = super::filename::parse(input)?;
            let path = workspace.absolute(&name);
            if workspace
                .doc(id)
                .ok_or(FileError::NoPath)?
                .path
                .as_ref()
                .is_some_and(|current| *current != path)
            {
                return Err(Error::Ex(
                    "writing another file is not supported yet".into(),
                ));
            }
            let unnamed = workspace.doc(id).is_some_and(|doc| doc.path.is_none());
            drop(workspace);
            if unnamed {
                self.rename_buffer(id, &name)?;
            }
        }
        let mut workspace = self.workspace.borrow_mut();
        let doc = workspace.doc_mut(id).ok_or(FileError::NoPath)?;
        if !force && !doc.new_name {
            doc.check_disk();
        }
        if !force && doc.disk_mark() == Some(DiskMark::Changed) {
            return Err(Error::Ex(
                "file changed on disk (use :w! to override)".into(),
            ));
        }
        let path = doc.path.clone().ok_or(FileError::NoPath)?;
        if !force && doc.readonly() {
            return Err(FileError::Readonly.into());
        }
        drop(workspace);
        self.emit_io("BufWritePre", id)?;
        let workspace = self.workspace.borrow();
        if workspace
            .doc(id)
            .is_none_or(|doc| doc.path.as_ref() != Some(&path))
        {
            return Err(Error::Ex(
                "E203: Autocommands changed buffer or file".into(),
            ));
        }
        drop(workspace);
        self.workspace.borrow_mut().select_service(id);
        let mut workspace = self.workspace.borrow_mut();
        let doc = workspace.doc_mut(id).ok_or(FileError::NoPath)?;
        if force {
            doc.save_force()?;
        } else {
            doc.save()?;
        }
        drop(workspace);
        self.events.borrow_mut().push(super::Notification {
            buffer: id,
            event: crate::module::Event::NoteSave(path),
        });
        self.emit_io("BufWritePost", id)
    }
}
