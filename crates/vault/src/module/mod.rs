mod changed;
mod create;
mod lazy;

use std::cell::Ref;
use std::io;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use v_ide_editor::module::{Cx, Event, Manifest, Module, Registration, Trigger};
use v_ide_editor::path::rel_path;

use crate::index::Index;
use changed::changed;
use create::create;
use lazy::LazyIndex;

static MANIFEST: Manifest = Manifest {
    name: "vault",
    commands: &["pick", "new", "scan-hidden", "backlinks"],
    triggers: &[Trigger::Startup, Trigger::Event("note_create")],
};

pub struct VaultModule {
    index: Rc<LazyIndex>,
}

impl VaultModule {
    pub fn registration() -> Registration {
        Registration {
            manifest: &MANIFEST,
            make: || Box::new(Self::unloaded()),
        }
    }

    fn unloaded() -> Self {
        Self {
            index: Rc::new(LazyIndex::new(PathBuf::new())),
        }
    }

    pub fn root(&self) -> &Path {
        self.index.root()
    }

    pub fn index(&self) -> Result<Ref<'_, Index>, String> {
        self.index.get()
    }

    pub fn items(&self) -> Result<Vec<String>, String> {
        self.index.paths()
    }

    pub fn scan_hidden(&self) -> bool {
        self.index.scan_hidden()
    }

    pub fn set_scan_hidden(&mut self, value: bool) -> io::Result<()> {
        self.index.set_scan_hidden(value);
        if self.index.is_built() {
            self.reindex()
        } else {
            Ok(())
        }
    }

    pub fn reindex(&mut self) -> io::Result<()> {
        self.index.rebuild()
    }

    fn announce(&self, cx: &mut Cx<'_>) {
        if let Ok(index) = self.index.get() {
            cx.emit(changed(&index));
        }
    }

    fn create_note(&self, cx: &mut Cx<'_>, name: &str) -> Result<(), String> {
        let created = create(self.root(), name).map_err(|err| format!("create failed: {err}"))?;
        if let Some(note) = created {
            if note.fresh {
                cx.emit(Event::NoteCreate(note.path.clone()));
            }
            cx.open(note.path);
        }
        Ok(())
    }

    fn toggle_hidden(&mut self, cx: &mut Cx<'_>, arg: &str) {
        let value = match arg {
            "on" => true,
            "off" => false,
            "toggle" => !self.scan_hidden(),
            _ => return cx.message("scan-hidden: on|off|toggle"),
        };
        let state = if value { "on" } else { "off" };
        let built = self.index.is_built();
        match self.set_scan_hidden(value) {
            Ok(()) => {
                if built {
                    self.announce(cx);
                }
                cx.message(format!("scan hidden folders: {state}"));
            }
            Err(err) => cx.message(format!("reindex failed: {err}")),
        }
    }

    fn backlinks(&self, cx: &mut Cx<'_>) -> Result<(), String> {
        let rel = cx
            .path()
            .map(|path| rel_path(self.root(), path))
            .unwrap_or_default();
        let rows: Vec<String> = self
            .index()?
            .backlinks(&rel)
            .iter()
            .map(|n| n.path.clone())
            .collect();
        if rows.is_empty() {
            cx.message("no notes link here");
        } else {
            cx.list("backlinks", rows, "e");
        }
        Ok(())
    }
}

impl Module for VaultModule {
    fn activate(&mut self, cx: &mut Cx<'_>) -> Result<(), String> {
        self.index = Rc::new(LazyIndex::new(cx.cwd().to_path_buf()));
        cx.provide_vault(self.index.clone());
        Ok(())
    }

    fn command(&mut self, cx: &mut Cx<'_>, name: &str, args: &str) -> Result<(), String> {
        match name {
            "pick" => cx.pick(self.items()?, "e"),
            "new" => self.create_note(cx, args)?,
            "scan-hidden" => self.toggle_hidden(cx, args.trim()),
            "backlinks" => self.backlinks(cx)?,
            _ => {}
        }
        Ok(())
    }

    fn event(&mut self, cx: &mut Cx<'_>, ev: &Event) {
        if let Event::NoteCreate(_) = ev
            && self.index.is_built()
            && self.reindex().is_ok()
        {
            self.announce(cx);
        }
    }
}
