use super::{Context, Error};
use crate::{
    autocmd::{EventName, GroupFilter, Names, Occurrence, Source},
    workspace::DocId,
};

impl Context<'_> {
    pub(super) fn emit_io(&self, event: &str, id: DocId) -> Result<(), Error> {
        let name = self
            .workspace
            .borrow()
            .doc(id)
            .map(|doc| doc.name.clone())
            .unwrap_or_default();
        self.emit_event(event, id, (!name.is_empty()).then_some(name.as_str()))
    }

    pub fn emit_filetype(&self, id: DocId) -> Result<(), Error> {
        let (file, filetype) = self
            .workspace
            .borrow()
            .doc(id)
            .map(|doc| (doc.name.clone(), doc.filetype().map(str::to_owned)))
            .unwrap_or_default();
        if let Some(filetype) = filetype {
            self.dispatch_names(
                EventName::parse("FileType")?,
                id,
                Names {
                    file,
                    short: filetype.clone(),
                    full: filetype,
                },
            )?;
        }
        Ok(())
    }

    pub fn emit_event(
        &self,
        name: &str,
        buffer: DocId,
        pattern: Option<&str>,
    ) -> Result<(), Error> {
        let event = EventName::parse(name)?;
        let Some(names) = self.autocmd_names(event, buffer, pattern) else {
            return Ok(());
        };
        self.dispatch_names(event, buffer, names)
    }

    pub fn emit_insert_change(&self, buffer: DocId) -> Result<(), Error> {
        let result = self.emit_event("TextChangedI", buffer, None);
        let mut workspace = self.workspace.borrow_mut();
        let cursor = workspace.current().editor.cursor();
        if workspace.select_service(buffer) {
            workspace.current_mut().editor.set_cursor(cursor);
        }
        result
    }

    fn dispatch_names(&self, event: EventName, buffer: DocId, names: Names) -> Result<(), Error> {
        let Some(bus) = self.observer.and_then(|observer| observer.autocmd_bus()) else {
            return Ok(());
        };
        let before = self.error_count();
        bus.dispatch(
            self,
            &Occurrence::new(event, Some(buffer), names),
            GroupFilter::All,
            Source::AutomaticEx,
        );
        if self.error_count() != before {
            return Err(Error::Autocmd(self.last_error.borrow().clone()));
        }
        Ok(())
    }
}
