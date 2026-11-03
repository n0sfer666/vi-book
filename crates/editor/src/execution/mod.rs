mod core;
mod diagnostics;
pub use diagnostics::DiagnosticCheckpoint;
mod event;
mod ex;
mod io;
mod modeline;
mod names;
mod option;
mod set;

use std::cell::{Cell, RefCell};

use crate::{
    options,
    workspace::{DocId, Workspace},
};

pub use option::{Observer, OptionSet};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    File(#[from] crate::Error),
    #[error(transparent)]
    Registry(#[from] crate::autocmd::Error),
    #[error("{0}")]
    Ex(String),
    #[error(transparent)]
    Option(#[from] options::Error),
    #[error("not an editor command: {0}")]
    Command(String),
    #[error("{0}")]
    Autocmd(String),
}

pub struct Context<'a> {
    diagnostic_context: u64,
    navigated: Cell<bool>,
    workspace: RefCell<&'a mut Workspace>,
    observer: Option<&'a dyn Observer>,
    option_active: Cell<bool>,
    modeline_active: Cell<bool>,
    ex_depth: Cell<usize>,
    messages: RefCell<Vec<String>>,
    errors: Cell<usize>,
    last_error: RefCell<String>,
}

impl<'a> Context<'a> {
    pub fn new(workspace: &'a mut Workspace) -> Self {
        Self {
            diagnostic_context: diagnostics::new_context(),
            navigated: Cell::new(false),
            workspace: RefCell::new(workspace),
            observer: None,
            option_active: Cell::new(false),
            modeline_active: Cell::new(false),
            ex_depth: Cell::new(0),
            messages: RefCell::new(Vec::new()),
            errors: Cell::new(0),
            last_error: RefCell::new(String::new()),
        }
    }

    pub fn with_observer(workspace: &'a mut Workspace, observer: &'a dyn Observer) -> Self {
        Self {
            observer: Some(observer),
            ..Self::new(workspace)
        }
    }

    pub fn option(&self, name: options::Name, access: options::Access) -> options::Value {
        self.workspace.borrow().option(name, access)
    }

    pub fn snapshot(&self) -> options::Snapshot {
        options::Snapshot::capture(&self.workspace.borrow())
    }

    pub fn buffer(&self, number: u64) -> Option<DocId> {
        let workspace = self.workspace.borrow();
        if number == 0 {
            return Some(workspace.current_id());
        }
        let id = DocId(number - 1);
        workspace.doc(id).map(|_| id)
    }

    pub fn buffer_number(&self, id: DocId) -> Option<u64> {
        self.workspace.borrow().doc(id)?;
        id.0.checked_add(1)
    }

    pub fn execute(&self, input: &str) -> Result<(), Error> {
        self.execute_ex(input)
    }

    pub fn navigated(&self) -> bool {
        self.navigated.get()
    }

    pub fn diagnostic_checkpoint(&self) -> DiagnosticCheckpoint {
        DiagnosticCheckpoint {
            context: self.diagnostic_context,
            serial: self.errors.get(),
        }
    }

    pub(crate) fn error_count(&self) -> usize {
        self.errors.get()
    }

    pub(crate) fn record_error(&self, error: String) {
        self.errors.set(self.errors.get().saturating_add(1));
        self.last_error.replace(error.clone());
        self.messages.borrow_mut().push(error);
    }

    pub fn take_messages(&self) -> Vec<String> {
        std::mem::take(&mut self.messages.borrow_mut())
    }
}
