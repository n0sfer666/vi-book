use super::{autocmd::events, lex::word};
use crate::{
    autocmd::{GroupFilter, Occurrence, Source},
    execution::{Context, Error},
};

impl Context<'_> {
    pub(super) fn ex_doautocmd(&self, input: &str) -> Result<(), Error> {
        let (first, tail) = word(input);
        let (modeline, input) = if first == "<nomodeline>" {
            (false, tail)
        } else {
            (true, input)
        };
        let (first, tail) = word(input);
        let bus = self.autocmd_bus()?;
        let (group, event_text, file) = if let Some(id) = bus.group(first) {
            let (event, file) = word(tail);
            (GroupFilter::Group(id), event, file)
        } else {
            (GroupFilter::All, first, tail)
        };
        if event_text == "*" {
            return Err(Error::Ex(
                "E217: Can't execute autocommands for ALL events".into(),
            ));
        }
        let events = events(event_text)?;
        let before = self.errors.get();
        let mut called = 0;
        for event in events {
            let buffer = self
                .buffer(0)
                .ok_or_else(|| Error::Ex("no current buffer".into()))?;
            let names = self
                .autocmd_names(event, buffer, (!file.is_empty()).then_some(file))
                .ok_or_else(|| Error::Ex("invalid event buffer".into()))?;
            called += bus.dispatch(
                self,
                &Occurrence::new(event, Some(buffer), names),
                group,
                Source::Ex,
            );
            if self.errors.get() != before {
                break;
            }
        }
        if called == 0 && self.errors.get() == before {
            self.messages
                .borrow_mut()
                .push(format!("No matching autocommands: {input}"));
        }
        if modeline && called > 0 {
            self.process_modelines();
        }
        if self.errors.get() != before {
            return Err(Error::Autocmd(self.last_error.borrow().clone()));
        }
        Ok(())
    }
}
