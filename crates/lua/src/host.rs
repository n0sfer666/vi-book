mod keymaps;

use std::cell::RefCell;
use std::rc::Rc;

use mlua::{Function, Table};
use v_ide_editor::module::VaultQuery;

use crate::config::{Bridge, ColorSpec, PRELUDE, Shared};
use crate::error::Error;
use crate::install::{install_buf, install_open, install_vault};
use crate::runtime::Runtime;

pub struct PluginHost {
    rt: Runtime,
    bridge: Shared,
}

impl PluginHost {
    pub fn new() -> Result<Self, Error> {
        let rt = Runtime::new()?;
        rt.exec(PRELUDE)?;
        let bridge: Shared = Rc::new(RefCell::new(Bridge::default()));
        crate::options::install(&rt, &bridge)?;
        install_buf(&rt, &bridge)?;
        install_vault(&rt, &bridge)?;
        install_open(&rt, &bridge)?;
        Ok(Self { rt, bridge })
    }

    pub fn load(&self, script: &str) -> Result<(), Error> {
        self.rt.exec(script)
    }

    pub fn set_options(&self, snapshot: v_ide_editor::options::Snapshot) {
        let mut state = self.bridge.borrow_mut();
        state.options.current = snapshot;
        state.options.pending.clear();
    }

    pub fn take_option_changes(
        &self,
    ) -> Vec<(v_ide_editor::options::Access, v_ide_editor::options::Change)> {
        std::mem::take(&mut self.bridge.borrow_mut().options.pending)
    }

    pub fn set_buffer(&self, text: &str, cursor: usize) {
        let mut b = self.bridge.borrow_mut();
        b.buf_text = text.to_string();
        b.buf_cursor = cursor;
    }

    pub fn buffer_text(&self) -> String {
        self.bridge.borrow().buf_text.clone()
    }

    pub fn set_vault(&self, vault: Rc<dyn VaultQuery>) {
        self.bridge.borrow_mut().vault = Some(vault);
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

    pub fn take_external_opens(&self) -> Vec<String> {
        std::mem::take(&mut self.bridge.borrow_mut().external_opens)
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

    pub fn menu_items(&self) -> Vec<(String, String, String)> {
        let Ok(menu) = self.vibook_field("__menu") else {
            return Vec::new();
        };
        menu.sequence_values::<Table>()
            .filter_map(Result::ok)
            .filter_map(|t| Some((t.get("key").ok()?, t.get("label").ok()?, t.get("rhs").ok()?)))
            .collect()
    }

    pub fn colors(&self) -> Vec<ColorSpec> {
        let Ok(list) = self.vibook_field("__colors") else {
            return Vec::new();
        };
        list.sequence_values::<Table>()
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let name: String = entry.get("name").ok()?;
                let spec: Table = entry.get("spec").ok()?;
                let flag = |key| {
                    spec.get::<Option<bool>>(key)
                        .ok()
                        .flatten()
                        .unwrap_or(false)
                };
                Some(ColorSpec {
                    name,
                    fg: spec.get::<Option<String>>("fg").ok().flatten(),
                    bold: flag("bold"),
                    italic: flag("italic"),
                    underline: flag("underline"),
                    dim: flag("dim"),
                })
            })
            .collect()
    }

    pub fn run_command(&self, name: &str) -> Result<bool, Error> {
        let commands = self.vibook_field("__commands")?;
        let Some(func) = commands.get::<Option<Function>>(name)? else {
            return Ok(false);
        };
        self.rt.reset_budget();
        func.call::<()>(())?;
        Ok(true)
    }

    pub fn fire(&self, event: &str) -> Result<(), Error> {
        let handlers = self.vibook_field("__handlers")?;
        if let Some(list) = handlers.get::<Option<Table>>(event)? {
            for func in list.sequence_values::<Function>() {
                self.rt.reset_budget();
                func?.call::<()>(())?;
            }
        }
        Ok(())
    }

    pub fn fire_with(&self, event: &str, arg: &str) -> Result<(), Error> {
        let handlers = self.vibook_field("__handlers")?;
        if let Some(list) = handlers.get::<Option<Table>>(event)? {
            for func in list.sequence_values::<Function>() {
                self.rt.reset_budget();
                func?.call::<()>(arg.to_string())?;
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
