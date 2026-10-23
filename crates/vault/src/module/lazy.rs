use std::cell::{Cell, Ref, RefCell};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use v_ide_editor::module::VaultQuery;

use crate::index::Index;
use crate::names::Names;
use crate::note::Note;
use crate::scan::{scan_with, walk_paths};

use super::level::Level;
use super::reset::Reset;
use super::scope;
use super::watch::Watch;

pub struct LazyIndex {
    root: PathBuf,
    scan_hidden: Cell<bool>,
    names: RefCell<Option<Names>>,
    index: RefCell<Option<Index>>,
    watch: Watch,
}

impl LazyIndex {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            scan_hidden: Cell::new(false),
            names: RefCell::new(None),
            index: RefCell::new(None),
            watch: Watch::new(),
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

    pub fn watch(&self) -> &Watch {
        &self.watch
    }

    pub fn rebuild(&self) -> io::Result<()> {
        self.watch.start(&self.root);
        let built = scan_with(&self.root, self.scan_hidden.get()).map(Index::from_notes);
        if let Ok(mut names) = self.names.try_borrow_mut() {
            *names = None;
        }
        let mut index = self.index.try_borrow_mut().map_err(io::Error::other)?;
        match built {
            Ok(fresh) => {
                *index = Some(fresh);
                self.watch.rescanned();
                Ok(())
            }
            Err(err) => {
                *index = None;
                Err(err)
            }
        }
    }

    pub fn upsert(&self, abs: &Path) -> io::Result<bool> {
        let Some(rel) = scope::member(&self.root, self.scan_hidden.get(), abs) else {
            return Ok(false);
        };
        if !self.is_built() {
            self.edit_names(|names| names.insert(rel))?;
            return Ok(false);
        }
        let mut index = self.index.try_borrow_mut().map_err(io::Error::other)?;
        match fs::read_to_string(abs) {
            Ok(source) => Ok(index
                .as_mut()
                .is_some_and(|index| index.upsert(Note::from_source(&rel, &source)))),
            Err(err) => {
                *index = None;
                drop(index);
                self.forget_names()?;
                Err(err)
            }
        }
    }

    pub fn edit<R>(&self, edit: impl FnOnce(Level<'_>) -> R) -> io::Result<Option<R>> {
        let mut index = self.index.try_borrow_mut().map_err(io::Error::other)?;
        if let Some(index) = index.as_mut() {
            return Ok(Some(edit(Level::Facts(index))));
        }
        drop(index);
        let mut names = self.names.try_borrow_mut().map_err(io::Error::other)?;
        Ok(names.as_mut().map(|names| edit(Level::Names(names))))
    }

    fn edit_names(&self, edit: impl FnOnce(&mut Names)) -> io::Result<()> {
        if let Some(names) = self
            .names
            .try_borrow_mut()
            .map_err(io::Error::other)?
            .as_mut()
        {
            edit(names);
        }
        Ok(())
    }

    pub fn reset(&self) -> io::Result<Reset> {
        let mut index = match self.index.try_borrow_mut() {
            Ok(index) => index,
            Err(err) => return Ok(Reset::Busy(err.to_string())),
        };
        if index.is_none() {
            drop(index);
            return Ok(self
                .forget_names()
                .map_or_else(|err| Reset::Busy(err.to_string()), |()| Reset::Forgot));
        }
        *index = Some(scan_with(&self.root, self.scan_hidden.get()).map(Index::from_notes)?);
        drop(index);
        if let Ok(mut names) = self.names.try_borrow_mut() {
            *names = None;
        }
        Ok(Reset::Rebuilt)
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
            self.watch.start(&self.root);
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
