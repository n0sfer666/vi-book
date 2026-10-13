use std::path::{Path, PathBuf};

use crate::module::{Event, Request};
use crate::workspace::Workspace;

pub struct Cx<'a> {
    ws: &'a Workspace,
    requests: Vec<Request>,
}

impl<'a> Cx<'a> {
    pub(crate) fn new(ws: &'a Workspace) -> Self {
        Self {
            ws,
            requests: Vec::new(),
        }
    }

    pub(crate) fn into_requests(self) -> Vec<Request> {
        self.requests
    }

    pub fn path(&self) -> Option<&Path> {
        self.ws.current().path.as_deref()
    }

    pub fn text(&self) -> String {
        self.ws.current().editor.text()
    }

    pub fn open(&mut self, path: PathBuf) {
        self.requests.push(Request::Open(path));
    }

    pub fn message(&mut self, msg: impl Into<String>) {
        self.requests.push(Request::Message(msg.into()));
    }

    pub fn pick(&mut self, items: Vec<String>, command: &str) {
        self.requests.push(Request::Pick {
            items,
            command: command.to_string(),
        });
    }

    pub fn list(&mut self, title: &str, rows: Vec<String>, command: &str) {
        self.requests.push(Request::List {
            title: title.to_string(),
            rows,
            command: command.to_string(),
        });
    }

    pub fn emit(&mut self, ev: Event) {
        self.requests.push(Request::Emit(ev));
    }
}
