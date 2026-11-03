use super::{Context, Error};
use crate::{
    autocmd::{EventName, GroupFilter, Occurrence, Source},
    workspace::DocId,
};

impl Context<'_> {
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
