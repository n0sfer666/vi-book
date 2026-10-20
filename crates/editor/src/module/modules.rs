use std::any::Any;
use std::collections::HashMap;
use std::rc::Rc;

use crate::module::entry::Entry;
use crate::module::{Cx, Event, Module, Registration, Request, VaultQuery};
use crate::workspace::Workspace;

pub struct Modules {
    entries: Vec<Entry>,
    last: HashMap<&'static str, Event>,
    vault: Option<Rc<dyn VaultQuery>>,
}

impl Modules {
    pub fn new(registrations: Vec<Registration>) -> Self {
        Self {
            entries: registrations.into_iter().map(Entry::new).collect(),
            last: HashMap::new(),
            vault: None,
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
        self.activate_where(ws, Entry::on_startup)
    }

    pub fn activate(&mut self, ws: &Workspace, name: &str) -> Vec<Request> {
        self.activate_where(ws, |entry| entry.named(name))
    }

    pub fn filetype(&mut self, ws: &Workspace, filetype: &str) -> Vec<Request> {
        self.activate_where(ws, |entry| entry.on_filetype(filetype))
    }

    pub fn command(&mut self, ws: &Workspace, name: &str, args: &str) -> Option<Vec<Request>> {
        let index = self
            .entries
            .iter()
            .position(|e| e.declares(name))
            .or_else(|| self.entries.iter().position(|e| e.provides(name)))?;
        let entry = &mut self.entries[index];
        let mut cx = Cx::new(ws, self.vault.clone());
        if let Some(err) = entry.failure() {
            cx.message(err);
        } else if let Some(module) = entry.active(&mut cx, &self.last)
            && let Err(err) = module.command(&mut cx, name, args)
        {
            cx.message(err);
        }
        Some(self.finish(cx))
    }

    pub fn event(&mut self, ws: &Workspace, ev: &Event) -> Vec<Request> {
        let mut cx = Cx::new(ws, self.vault.clone());
        let name = ev.name();
        for entry in self.entries.iter_mut() {
            let module = if entry.listens(name) {
                entry.active(&mut cx, &self.last)
            } else if entry.observes(name) {
                entry.module_mut()
            } else {
                None
            };
            if let Some(module) = module {
                module.event(&mut cx, ev);
            }
        }
        if self.entries.iter().any(|e| e.observes(name)) {
            self.last.insert(name, ev.clone());
        }
        self.finish(cx)
    }

    fn activate_where(&mut self, ws: &Workspace, pick: impl Fn(&Entry) -> bool) -> Vec<Request> {
        let mut cx = Cx::new(ws, self.vault.clone());
        for entry in self.entries.iter_mut().filter(|e| pick(e)) {
            entry.active(&mut cx, &self.last);
        }
        self.finish(cx)
    }

    fn finish(&mut self, cx: Cx<'_>) -> Vec<Request> {
        let (requests, vault) = cx.into_parts();
        self.vault = vault;
        requests
    }
}
