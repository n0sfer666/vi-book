use std::any::Any;

use crate::module::entry::Entry;
use crate::module::{Cx, Event, Module, Registration, Request};
use crate::workspace::Workspace;

pub struct Modules {
    entries: Vec<Entry>,
}

impl Modules {
    pub fn new(registrations: Vec<Registration>) -> Self {
        Self {
            entries: registrations.into_iter().map(Entry::new).collect(),
        }
    }

    pub fn get<T: Module>(&self) -> Option<&T> {
        self.entries.iter().find_map(|entry| {
            let any: &dyn Any = entry.module()?;
            any.downcast_ref::<T>()
        })
    }

    pub fn get_mut<T: Module>(&mut self) -> Option<&mut T> {
        self.entries.iter_mut().find_map(|entry| {
            let any: &mut dyn Any = entry.module_mut()?;
            any.downcast_mut::<T>()
        })
    }

    pub fn startup(&mut self, ws: &Workspace) -> Vec<Request> {
        let mut cx = Cx::new(ws);
        for entry in self.entries.iter_mut().filter(|e| e.on_startup()) {
            entry.active(&mut cx);
        }
        cx.into_requests()
    }

    pub fn command(&mut self, ws: &Workspace, name: &str, args: &str) -> Option<Vec<Request>> {
        let index = self
            .entries
            .iter()
            .position(|e| e.declares(name))
            .or_else(|| self.entries.iter().position(|e| e.provides(name)))?;
        let entry = &mut self.entries[index];
        let mut cx = Cx::new(ws);
        if let Some(module) = entry.active(&mut cx)
            && let Err(err) = module.command(&mut cx, name, args)
        {
            cx.message(err);
        }
        Some(cx.into_requests())
    }

    pub fn event(&mut self, ws: &Workspace, ev: &Event) -> Vec<Request> {
        let mut cx = Cx::new(ws);
        for entry in self.entries.iter_mut().filter(|e| e.listens(ev.name())) {
            if let Some(module) = entry.active(&mut cx) {
                module.event(&mut cx, ev);
            }
        }
        cx.into_requests()
    }
}
