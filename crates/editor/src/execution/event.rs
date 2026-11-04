use super::{Context, Error};
use crate::{
    autocmd::{EventName, GroupFilter, Occurrence, Source},
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

    pub fn emit_event(
        &self,
        name: &str,
        buffer: DocId,
        pattern: Option<&str>,
    ) -> Result<(), Error> {
        let Some(bus) = self.observer.and_then(|observer| observer.autocmd_bus()) else {
            return Ok(());
        };
        let event = EventName::parse(name)?;
        let Some(names) = self.autocmd_names(event, buffer, pattern) else {
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
