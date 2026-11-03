use super::lex::word;
use crate::{
    autocmd::{Action, Attributes, EventName, Filter, GroupFilter, Selector, patterns},
    execution::{Context, Error},
};
use std::rc::Rc;

pub(super) fn events(text: &str) -> Result<Vec<EventName>, Error> {
    if text == "*" {
        return Ok(EventName::all().collect());
    }
    if text.is_empty() {
        return Err(Error::Ex("E471: Argument required".into()));
    }
    text.split(',')
        .map(|event| {
            EventName::parse(event)
                .map_err(|_| Error::Ex(format!("E216: No such group or event: {event}")))
        })
        .collect()
}

impl Context<'_> {
    pub(super) fn ex_autocmd(&self, input: &str, bang: bool) -> Result<(), Error> {
        let bus = self.autocmd_bus()?;
        let (first, tail) = word(input);
        let (explicit, event_text, tail) = if let Some(id) = bus.group(first) {
            let (event, tail) = word(tail);
            (Some(id), event, tail)
        } else {
            (None, first, tail)
        };
        let names = if event_text.is_empty() {
            None
        } else {
            Some(events(event_text)?)
        };
        let (pattern, mut action) = word(tail);
        let mut attributes = Attributes {
            group: explicit.or(bus.current_group()),
            ..Attributes::default()
        };
        loop {
            let (flag, rest) = word(action);
            match flag {
                "++once" => {
                    if attributes.once {
                        return Err(Error::Ex("E983: Duplicate argument: ++once".into()));
                    }
                    attributes.once = true;
                }
                "++nested" | "nested" => {
                    if attributes.nested {
                        return Err(Error::Ex("E983: Duplicate argument: ++nested".into()));
                    }
                    attributes.nested = true;
                }
                _ => break,
            }
            action = rest;
        }
        let selectors: Vec<_> = patterns(pattern)
            .map(|source| self.autocmd_selector(source, bus.event_buffer()))
            .collect();
        let group = if bang || !action.is_empty() {
            attributes
                .group
                .map_or(GroupFilter::Default, GroupFilter::Group)
        } else {
            explicit.map_or(GroupFilter::All, GroupFilter::Group)
        };
        if !action.is_empty() {
            if event_text == "*" {
                return Err(Error::Ex(
                    "E1155: Cannot define autocommands for ALL events".into(),
                ));
            }
            let names = names.ok_or_else(|| Error::Ex("E471: Argument required".into()))?;
            for event in names {
                for selector in &selectors {
                    if bang {
                        let selector = selector.as_ref().map_err(Clone::clone)?;
                        bus.clear(&Filter {
                            group,
                            events: Some(vec![event]),
                            selectors: (!matches!(selector,Selector::File(p)if p=="*"))
                                .then(|| vec![selector.clone()]),
                            id: None,
                        });
                    }
                    bus.define_ex_checked(
                        attributes.clone(),
                        Action::Command(Rc::from(action)),
                        &[event.as_str()],
                        std::slice::from_ref(selector),
                    )?;
                }
            }
            return Ok(());
        }
        let selectors = if pattern.is_empty() || pattern == "*" {
            None
        } else {
            Some(selectors.into_iter().collect::<Result<Vec<_>, _>>()?)
        };
        let filter = Filter {
            group,
            events: names,
            selectors,
            id: None,
        };
        if bang {
            bus.clear(&filter);
        } else {
            self.ex_listing(&filter)?;
        }
        Ok(())
    }
}
