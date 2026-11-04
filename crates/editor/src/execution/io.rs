use std::io::ErrorKind;

use super::{Context, Error};
use crate::{
    Error as FileError,
    document::Document,
    options::{Access, Name},
};

impl Context<'_> {
    pub(super) fn edit_buffer(&self, input: &str, force: bool) -> Result<(), Error> {
        let (path, existing, current, dirty) = {
            let workspace = self.workspace.borrow();
            let doc = workspace.current();
            let path = if input.is_empty() {
                doc.path.clone().ok_or(FileError::NoPath)?
            } else {
                workspace.absolute(&super::filename::parse(input)?)
            };
            let existing = workspace.find_path(&path);
            (path, existing, workspace.current_id(), doc.is_dirty())
        };
        if existing == Some(current) && dirty && !force {
            return Err(Error::Ex(
                "E37: No write since last change (add ! to override)".into(),
            ));
        }
        if let Some(id) = existing {
            let mut workspace = self.workspace.borrow_mut();
            if id == current {
                let doc = workspace
                    .doc_mut(id)
                    .ok_or_else(|| Error::Ex("invalid edit buffer".into()))?;
                match doc.reload() {
                    Err(FileError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                        doc.reset_new_file()
                    }
                    result => result?,
                }
            }
            let removed = if id != current {
                workspace.select(id);
                workspace.doc(current).is_none().then_some(current)
            } else {
                None
            };
            drop(workspace);
            if let Some(id) = removed
                && let Ok(bus) = self.autocmd_bus()
            {
                bus.forget_buffer(id);
            }
            return Ok(());
        }
        let mut doc = match Document::open(&path) {
            Ok(doc) => doc,
            Err(FileError::Io(error)) if error.kind() == ErrorKind::NotFound => {
                let mut doc = Document::new("");
                doc.path = Some(path);
                doc.new_name = true;
                doc
            }
            Err(error) => return Err(error.into()),
        };
        let mut workspace = self.workspace.borrow_mut();
        doc.name = if input.is_empty() {
            String::new()
        } else {
            super::filename::parse(input)?
        };
        doc.options.set(
            Name::Modeline,
            workspace.option(Name::Modeline, Access::Global),
        );
        let id = workspace.add(doc);
        workspace.select(id);
        let removed = workspace.doc(current).is_none().then_some(current);
        drop(workspace);
        if let Some(id) = removed
            && let Ok(bus) = self.autocmd_bus()
        {
            bus.forget_buffer(id);
        }
        Ok(())
    }
}
