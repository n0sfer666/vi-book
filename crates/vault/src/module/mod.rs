mod create;

use std::io;
use std::path::{Path, PathBuf};

use v_ide_editor::module::{Cx, Event, Manifest, Module, Registration, Trigger};

use crate::index::Index;
use crate::scan::{rel_path, scan_with};
use create::create;

static MANIFEST: Manifest = Manifest {
    name: "vault",
    commands: &["pick", "new", "scan-hidden", "backlinks"],
    triggers: &[Trigger::Startup],
};

pub struct VaultModule {
    root: PathBuf,
    index: Index,
    scan_hidden: bool,
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
            root: PathBuf::new(),
            index: Index::from_notes(Vec::new()),
            scan_hidden: false,
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn index(&self) -> &Index {
        &self.index
    }

    pub fn items(&self) -> Vec<String> {
        self.index.notes().iter().map(|n| n.path.clone()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.index.notes().is_empty()
    }

    pub fn scan_hidden(&self) -> bool {
        self.scan_hidden
    }

    pub fn set_scan_hidden(&mut self, value: bool) -> io::Result<()> {
        self.scan_hidden = value;
        self.reindex()
    }

    pub fn reindex(&mut self) -> io::Result<()> {
        self.index = Index::from_notes(scan_with(&self.root, self.scan_hidden)?);
        Ok(())
    }

    fn create_note(&mut self, cx: &mut Cx<'_>, name: &str) -> Result<(), String> {
        let created = create(&self.root, name).map_err(|err| format!("create failed: {err}"))?;
        if let Some(path) = created {
            let _ = self.reindex();
            cx.open(path);
        }
        Ok(())
    }

    fn toggle_hidden(&mut self, cx: &mut Cx<'_>, arg: &str) {
        let value = match arg {
            "on" => true,
            "off" => false,
            "toggle" => !self.scan_hidden,
            _ => return cx.message("scan-hidden: on|off|toggle"),
        };
        let state = if value { "on" } else { "off" };
        match self.set_scan_hidden(value) {
            Ok(()) => cx.message(format!("scan hidden folders: {state}")),
            Err(err) => cx.message(format!("reindex failed: {err}")),
        }
    }

    fn backlinks(&self, cx: &mut Cx<'_>) {
        let rel = cx
            .path()
            .map(|path| rel_path(&self.root, path))
            .unwrap_or_default();
        let rows: Vec<String> = self
            .index
            .backlinks(&rel)
            .iter()
            .map(|n| n.path.clone())
            .collect();
        if rows.is_empty() {
            cx.message("no notes link here");
        } else {
            cx.list("backlinks", rows, "e");
        }
    }
}

impl Module for VaultModule {
    fn activate(&mut self, cx: &mut Cx<'_>) -> Result<(), String> {
        self.root = cx.cwd().to_path_buf();
        self.reindex().map_err(|err| format!("vault: {err}"))
    }

    fn command(&mut self, cx: &mut Cx<'_>, name: &str, args: &str) -> Result<(), String> {
        match name {
            "pick" => cx.pick(self.items(), "e"),
            "new" => self.create_note(cx, args)?,
            "scan-hidden" => self.toggle_hidden(cx, args.trim()),
            "backlinks" => self.backlinks(cx),
            _ => {}
        }
        Ok(())
    }

    fn event(&mut self, _cx: &mut Cx<'_>, _ev: &Event) {}
}
