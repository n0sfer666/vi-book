use std::io::ErrorKind;

use super::{Context, Error};
use crate::{
    Error as FileError,
    disk::DiskMark,
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
                workspace.absolute(&filename(input)?)
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
            filename(input)?
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

    pub(super) fn write_buffer(&self, input: &str, force: bool) -> Result<(), Error> {
        if !input.is_empty() {
            let workspace = self.workspace.borrow();
            let path = workspace.absolute(&filename(input)?);
            if workspace
                .current()
                .path
                .as_ref()
                .is_some_and(|current| *current != path)
            {
                return Err(Error::Ex(
                    "writing another file is not supported yet".into(),
                ));
            }
            let unnamed = workspace.current().path.is_none();
            drop(workspace);
            if unnamed {
                self.name_buffer(input)?;
            }
        }
        let mut workspace = self.workspace.borrow_mut();
        let doc = workspace.current_mut();
        if !force && !doc.new_name {
            doc.check_disk();
        }
        if !force && doc.disk_mark() == Some(DiskMark::Changed) {
            return Err(Error::Ex(
                "file changed on disk (use :w! to override)".into(),
            ));
        }
        if force {
            doc.save_force()?;
        } else {
            doc.save()?;
        }
        Ok(())
    }
}

fn filename(input: &str) -> Result<String, Error> {
    if input.starts_with(['%', '#'])
        || ["<afile>", "<amatch>", "<abuf>"]
            .iter()
            .any(|token| input.contains(token))
    {
        return Err(Error::Ex(
            "filename expansions are not supported yet".into(),
        ));
    }
    let mut out = String::new();
    let mut escape = false;
    for ch in input.chars() {
        if escape {
            out.push(ch);
            escape = false;
        } else if ch == '\\' {
            escape = true;
        } else if matches!(ch, ' ' | '\t') {
            return Err(Error::Ex("E172: Only one file name allowed".into()));
        } else {
            out.push(ch);
        }
    }
    if escape {
        out.push('\\');
    }
    Ok(out)
}
