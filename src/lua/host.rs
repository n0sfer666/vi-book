use std::cell::RefCell;
use std::rc::Rc;

use mlua::{Function, Table};

use crate::lua::config::{Bridge, Keymap, PRELUDE, Shared};
use crate::lua::error::Error;
use crate::lua::install::{install_buf, install_vault};
use crate::lua::runtime::Runtime;

pub struct PluginHost {
    rt: Runtime,
    bridge: Shared,
}

impl PluginHost {
    pub fn new() -> Result<Self, Error> {
        let rt = Runtime::new()?;
        rt.exec(PRELUDE)?;
        let bridge: Shared = Rc::new(RefCell::new(Bridge::default()));
        install_buf(&rt, &bridge)?;
        install_vault(&rt, &bridge)?;
        Ok(Self { rt, bridge })
    }

    pub fn load(&self, script: &str) -> Result<(), Error> {
        self.rt.exec(script)
    }

    pub fn set_buffer(&self, text: &str, cursor: usize) {
        let mut b = self.bridge.borrow_mut();
        b.buf_text = text.to_string();
        b.buf_cursor = cursor;
    }

    pub fn buffer_text(&self) -> String {
        self.bridge.borrow().buf_text.clone()
    }

    pub fn set_vault(&self, notes: Vec<String>, tags: Vec<String>) {
        let mut b = self.bridge.borrow_mut();
        b.vault_notes = notes;
        b.vault_tags = tags;
    }

    pub fn set_backlinks(&self, path: &str, sources: Vec<String>) {
        self.bridge
            .borrow_mut()
            .backlinks
            .insert(path.to_string(), sources);
    }

    pub fn creates(&self) -> Vec<String> {
        self.bridge.borrow().creates.clone()
    }

    pub fn opens(&self) -> Vec<String> {
        self.bridge.borrow().opens.clone()
    }

    pub fn take_opens(&self) -> Vec<String> {
        std::mem::take(&mut self.bridge.borrow_mut().opens)
    }

    pub fn take_creates(&self) -> Vec<String> {
        std::mem::take(&mut self.bridge.borrow_mut().creates)
    }

    pub fn opt_bool(&self, name: &str) -> Option<bool> {
        self.opt_table().ok()?.get(name).ok().flatten()
    }

    pub fn opt_string(&self, name: &str) -> Option<String> {
        self.opt_table().ok()?.get(name).ok().flatten()
    }

    pub fn command_names(&self) -> Vec<String> {
        let Ok(commands) = self.vibook_field("__commands") else {
            return Vec::new();
        };
        commands
            .pairs::<String, Function>()
            .filter_map(Result::ok)
            .map(|(name, _)| name)
            .collect()
    }

    pub fn keymaps(&self) -> Vec<Keymap> {
        let Ok(maps) = self.vibook_field("__keymaps") else {
            return Vec::new();
        };
        maps.sequence_values::<Table>()
            .filter_map(Result::ok)
            .filter_map(|t| {
                Some(Keymap {
                    mode: t.get("mode").ok()?,
                    lhs: t.get("lhs").ok()?,
                    rhs: t.get("rhs").ok()?,
                })
            })
            .collect()
    }

    pub fn run_command(&self, name: &str) -> Result<bool, Error> {
        let commands = self.vibook_field("__commands")?;
        let Some(func) = commands.get::<Option<Function>>(name)? else {
            return Ok(false);
        };
        func.call::<()>(())?;
        Ok(true)
    }

    pub fn fire(&self, event: &str) -> Result<(), Error> {
        let handlers = self.vibook_field("__handlers")?;
        if let Some(list) = handlers.get::<Option<Table>>(event)? {
            for func in list.sequence_values::<Function>() {
                func?.call::<()>(())?;
            }
        }
        Ok(())
    }

    fn vibook(&self) -> Result<Table, Error> {
        Ok(self.rt.lua().globals().get("vibook")?)
    }

    fn vibook_field(&self, field: &str) -> Result<Table, Error> {
        Ok(self.vibook()?.get(field)?)
    }

    fn opt_table(&self) -> Result<Table, Error> {
        self.vibook_field("opt")
    }
}
