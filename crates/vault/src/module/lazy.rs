use std::cell::{Cell, Ref, RefCell};
use std::io;
use std::path::{Path, PathBuf};

use v_ide_editor::module::VaultQuery;

use crate::index::Index;
use crate::scan::scan_with;

pub struct LazyIndex {
    root: PathBuf,
    scan_hidden: Cell<bool>,
    index: RefCell<Option<Index>>,
}

impl LazyIndex {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            scan_hidden: Cell::new(false),
            index: RefCell::new(None),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn scan_hidden(&self) -> bool {
        self.scan_hidden.get()
    }

    pub fn set_scan_hidden(&self, value: bool) {
        self.scan_hidden.set(value);
    }

    pub fn is_built(&self) -> bool {
        self.index.try_borrow().is_ok_and(|index| index.is_some())
    }

    pub fn rebuild(&self) -> io::Result<()> {
        let index = Index::from_notes(scan_with(&self.root, self.scan_hidden.get())?);
        *self.index.try_borrow_mut().map_err(io::Error::other)? = Some(index);
        Ok(())
    }

    pub fn get(&self) -> Result<Ref<'_, Index>, String> {
        if !self.is_built() {
            self.rebuild().map_err(|err| format!("vault: {err}"))?;
        }
        let index = self
            .index
            .try_borrow()
            .map_err(|err| format!("vault: {err}"))?;
        Ref::filter_map(index, Option::as_ref).map_err(|_| String::from("vault: not indexed"))
    }

    pub fn paths(&self) -> Result<Vec<String>, String> {
        Ok(self.get()?.notes().iter().map(|n| n.path.clone()).collect())
    }
}

impl VaultQuery for LazyIndex {
    fn notes(&self) -> Result<Vec<String>, String> {
        self.paths()
    }

    fn tags(&self) -> Result<Vec<String>, String> {
        Ok(self.get()?.tags())
    }

    fn backlinks(&self, path: &str) -> Result<Vec<String>, String> {
        let index = self.get()?;
        Ok(index
            .backlinks(path)
            .iter()
            .map(|n| n.path.clone())
            .collect())
    }
}
