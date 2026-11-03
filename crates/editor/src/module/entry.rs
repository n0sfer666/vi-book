use std::collections::HashMap;

use crate::module::{Cx, Event, Module, Registration, Trigger};

enum State {
    Idle,
    Active(Box<dyn Module>),
    Failed(String),
}

pub struct Entry {
    registration: Registration,
    state: State,
}

impl Entry {
    pub fn new(registration: Registration) -> Self {
        Self {
            registration,
            state: State::Idle,
        }
    }

    pub fn named(&self, name: &str) -> bool {
        self.registration.manifest.name == name
    }

    pub fn declares(&self, command: &str) -> bool {
        self.registration.manifest.commands.contains(&command)
    }

    pub fn provides(&self, command: &str) -> bool {
        self.module()
            .is_some_and(|module| module.commands().iter().any(|c| c == command))
    }

    pub fn listens(&self, event: &str) -> bool {
        self.triggers()
            .iter()
            .any(|t| matches!(t, Trigger::Event(name) if *name == event))
    }

    pub fn observes(&self, event: &str) -> bool {
        self.observed().any(|name| name == event)
    }

    pub fn on_startup(&self) -> bool {
        self.triggers().contains(&Trigger::Startup)
    }

    pub fn on_filetype(&self, filetype: &str) -> bool {
        self.triggers()
            .iter()
            .any(|t| matches!(t, Trigger::Filetype(ft) if *ft == filetype))
    }

    pub fn failure(&self) -> Option<&str> {
        match &self.state {
            State::Failed(err) => Some(err),
            _ => None,
        }
    }

    pub fn active(
        &mut self,
        cx: &mut Cx<'_>,
        last: &HashMap<&'static str, Event>,
    ) -> Option<&mut dyn Module> {
        if let State::Idle = self.state {
            let mut module = (self.registration.make)();
            self.state = match module.activate(cx) {
                Ok(()) => {
                    for ev in self.observed().filter_map(|name| last.get(name)) {
                        super::deliver(module.as_mut(), cx, ev);
                    }
                    State::Active(module)
                }
                Err(err) => {
                    cx.message(err.clone());
                    State::Failed(err)
                }
            };
        }
        self.module_mut()
    }

    pub fn module(&self) -> Option<&dyn Module> {
        match &self.state {
            State::Active(module) => Some(module.as_ref()),
            _ => None,
        }
    }

    pub fn module_mut(&mut self) -> Option<&mut dyn Module> {
        match &mut self.state {
            State::Active(module) => Some(module.as_mut()),
            _ => None,
        }
    }

    fn observed(&self) -> impl Iterator<Item = &'static str> {
        self.triggers().iter().filter_map(|t| match t {
            Trigger::Observe(name) => Some(*name),
            _ => None,
        })
    }

    fn triggers(&self) -> &'static [Trigger] {
        self.registration.manifest.triggers
    }
}
