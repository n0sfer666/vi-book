use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::module::{Event, Request, VaultQuery};
use crate::options::{Access, Change, Snapshot};
use crate::workspace::Workspace;

pub struct Cx<'a> {
    ws: &'a Workspace,
    requests: Vec<Request>,
    vault: Option<Rc<dyn VaultQuery>>,
}

impl<'a> Cx<'a> {
    pub(crate) fn new(ws: &'a Workspace, vault: Option<Rc<dyn VaultQuery>>) -> Self {
        Self {
            ws,
            requests: Vec::new(),
            vault,
        }
    }

    pub(crate) fn into_parts(self) -> (Vec<Request>, Option<Rc<dyn VaultQuery>>) {
        (self.requests, self.vault)
    }

    pub fn provide_vault(&mut self, vault: Rc<dyn VaultQuery>) {
        self.vault = Some(vault);
    }

    pub fn vault(&self) -> Option<Rc<dyn VaultQuery>> {
        self.vault.clone()
    }

    pub fn cwd(&self) -> &Path {
        self.ws.cwd()
    }

    pub fn path(&self) -> Option<&Path> {
        self.ws.current().path.as_deref()
    }

    pub fn text(&self) -> String {
        self.ws.current().editor.text()
    }

    pub fn cursor(&self) -> usize {
        self.ws.current().editor.text_index()
    }

    pub fn options(&self) -> Snapshot {
        Snapshot::capture(self.ws)
    }

    pub fn set_option(&mut self, access: Access, change: Change) {
        self.requests.push(Request::SetOption {
            doc: self.ws.current_id(),
            window: self.ws.active_window(),
            access,
            change,
        });
    }

    pub fn set_text(&mut self, text: String) {
        self.requests.push(Request::SetText(text));
    }

    pub fn open_external(&mut self, rel: String) {
        self.requests.push(Request::OpenExternal(rel));
    }

    pub fn command(&mut self, line: String) {
        self.requests.push(Request::Command(line));
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
