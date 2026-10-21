use std::cell::{Cell, Ref, RefCell};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

use v_ide_editor::module::VaultQuery;
use v_ide_editor::path::rel_path;

use crate::index::Index;
use crate::names::Names;
use crate::note::Note;
use crate::scan::{hidden, scan_with, walk_paths};

pub struct LazyIndex {
    root: PathBuf,
    scan_hidden: Cell<bool>,
    names: RefCell<Option<Names>>,
    index: RefCell<Option<Index>>,
}

impl LazyIndex {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            scan_hidden: Cell::new(false),
            names: RefCell::new(None),
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
        let built = scan_with(&self.root, self.scan_hidden.get()).map(Index::from_notes);
        if let Ok(mut names) = self.names.try_borrow_mut() {
            *names = None;
        }
        let mut index = self.index.try_borrow_mut().map_err(io::Error::other)?;
        match built {
            Ok(fresh) => {
                *index = Some(fresh);
                Ok(())
            }
            Err(err) => {
                *index = None;
                Err(err)
            }
        }
    }

    pub fn upsert(&self, abs: &Path) -> io::Result<bool> {
        let Some(rel) = self.member(abs) else {
            return Ok(false);
        };
        if !self.is_built() {
            if let Some(names) = self
                .names
                .try_borrow_mut()
                .map_err(io::Error::other)?
                .as_mut()
            {
                names.insert(rel);
            }
            return Ok(false);
        }
        let mut index = self.index.try_borrow_mut().map_err(io::Error::other)?;
        match fs::read_to_string(abs) {
            Ok(source) => {
                if let Some(index) = index.as_mut() {
                    index.upsert(Note::from_source(&rel, &source));
                }
                Ok(true)
            }
            Err(err) => {
                *index = None;
                drop(index);
                self.forget_names()?;
                Err(err)
            }
        }
    }

    fn member(&self, abs: &Path) -> Option<String> {
        let rel = abs.strip_prefix(&self.root).ok()?;
        let inside = rel.components().all(|c| match c {
            Component::Normal(name) => self.scan_hidden.get() || !hidden(name),
            _ => false,
        });
        let md = rel.extension().is_some_and(|ext| ext == "md");
        (inside && md).then(|| rel_path(&self.root, abs))
    }

    pub fn forget_names(&self) -> io::Result<()> {
        *self.names.try_borrow_mut().map_err(io::Error::other)? = None;
        Ok(())
    }

    pub fn names(&self) -> Result<Ref<'_, Names>, String> {
        if self.is_built() {
            return self.get().map(|index| Ref::map(index, Index::names));
        }
        if self.names.try_borrow().is_ok_and(|names| names.is_none()) {
            let paths = walk_paths(&self.root, self.scan_hidden.get())
                .map_err(|err| format!("vault: {err}"))?;
            *self
                .names
                .try_borrow_mut()
                .map_err(|err| format!("vault: {err}"))? = Some(Names::from_paths(paths));
        }
        let names = self
            .names
            .try_borrow()
            .map_err(|err| format!("vault: {err}"))?;
        Ref::filter_map(names, Option::as_ref).map_err(|_| String::from("vault: not indexed"))
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
        Ok(self.names()?.paths().to_vec())
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
