use crate::module::entry::Entry;
use crate::module::{Cx, Event, Registration, Request};
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

    pub fn startup(&mut self, ws: &Workspace) -> Vec<Request> {
        let mut cx = Cx::new(ws);
        for entry in self.entries.iter_mut().filter(|e| e.on_startup()) {
            entry.active(&mut cx);
        }
        cx.into_requests()
    }

    pub fn command(&mut self, ws: &Workspace, name: &str, args: &str) -> Option<Vec<Request>> {
        let entry = self.entries.iter_mut().find(|e| e.handles(name))?;
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
