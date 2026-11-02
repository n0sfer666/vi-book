use std::{collections::HashMap, rc::Rc};

use super::{Bus, State};
use crate::autocmd::{
    Action, Attributes, AutocmdId, Error, EventName, Pattern, Selector, patterns,
};

impl Bus {
    pub fn define(
        &self,
        attributes: Attributes,
        action: Action,
        events: &[&str],
        selectors: &[Selector],
    ) -> Result<AutocmdId, Error> {
        self.install(attributes, action, events, selectors.iter().map(Ok), true)
            .map(|id| id.unwrap())
    }

    pub fn define_checked(
        &self,
        attributes: Attributes,
        action: Action,
        events: &[&str],
        selectors: &[Result<Selector, Error>],
    ) -> Result<AutocmdId, Error> {
        self.install(
            attributes,
            action,
            events,
            selectors
                .iter()
                .map(|selector| selector.as_ref().map_err(Clone::clone)),
            true,
        )
        .map(|id| id.unwrap())
    }

    pub fn define_ex(
        &self,
        attributes: Attributes,
        action: Action,
        events: &[&str],
        selectors: &[Selector],
    ) -> Result<(), Error> {
        self.install(attributes, action, events, selectors.iter().map(Ok), false)
            .map(|_| ())
    }

    fn install<'a>(
        &self,
        attributes: Attributes,
        action: Action,
        events: &[&str],
        selectors: impl Clone + Iterator<Item = Result<&'a Selector, Error>>,
        api: bool,
    ) -> Result<Option<AutocmdId>, Error> {
        let _keepalive = action.clone();
        let mut state = self.state.borrow_mut();
        let State { registry, compiled } = &mut *state;
        let mut definition = if api {
            registry.define(attributes, action)?
        } else {
            registry.define_ex(attributes, action)?
        };
        let id = definition.id();
        let mut cache = HashMap::new();
        for source in events {
            let event = EventName::parse(source)?;
            for selector in selectors.clone() {
                let selector = selector?;
                match selector {
                    Selector::Buffer(_) => {
                        definition.push(event, selector.clone());
                    }
                    Selector::File(source) => {
                        for source in patterns(source) {
                            let pattern = if let Some(pattern) = cache.get(source) {
                                Rc::clone(pattern)
                            } else {
                                let pattern = Rc::new(Pattern::compile(source)?);
                                cache.insert(source.to_string(), pattern.clone());
                                pattern
                            };
                            let key = definition.push(event, Selector::File(source.into()));
                            compiled.insert(key, pattern);
                        }
                    }
                }
            }
        }
        Ok(id)
    }
}
