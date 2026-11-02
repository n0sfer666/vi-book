use std::cell::Cell;
use std::rc::Rc;

use mlua::{HookTriggers, Lua, LuaOptions, StdLib, VmState};

use crate::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Os,
    Io,
}

const STEP: u32 = 100_000;
const BUDGET: u64 = 50_000_000;

pub struct Runtime {
    lua: Lua,
    steps: Rc<Cell<u64>>,
    depth: Cell<usize>,
}

impl Runtime {
    pub fn new() -> Result<Self, Error> {
        let libs = StdLib::STRING | StdLib::TABLE | StdLib::MATH | StdLib::UTF8;
        let lua = Lua::new_with(libs, LuaOptions::default())?;
        for global in ["dofile", "loadfile", "load", "loadstring", "print", "warn"] {
            lua.globals().set(global, mlua::Value::Nil)?;
        }
        let steps = Rc::new(Cell::new(0u64));
        let counter = steps.clone();
        lua.set_hook(
            HookTriggers::new().every_nth_instruction(STEP),
            move |_, _| {
                let used = counter.get() + STEP as u64;
                counter.set(used);
                if used > BUDGET {
                    Err(mlua::Error::RuntimeError(
                        "plugin exceeded instruction budget".into(),
                    ))
                } else {
                    Ok(VmState::Continue)
                }
            },
        )?;
        Ok(Self {
            lua,
            steps,
            depth: Cell::new(0),
        })
    }

    pub fn grant(&self, cap: Capability) -> Result<(), Error> {
        let lib = match cap {
            Capability::Os => StdLib::OS,
            Capability::Io => StdLib::IO,
        };
        self.lua.load_std_libs(lib)?;
        Ok(())
    }

    pub fn exec(&self, script: &str) -> Result<(), Error> {
        let _invocation = self.enter();
        self.lua.load(script).exec()?;
        Ok(())
    }

    pub fn eval_bool(&self, expr: &str) -> Result<bool, Error> {
        let _invocation = self.enter();
        Ok(self.lua.load(expr).eval()?)
    }

    pub fn eval_string(&self, expr: &str) -> Result<String, Error> {
        let _invocation = self.enter();
        Ok(self.lua.load(expr).eval()?)
    }

    pub(crate) fn enter(&self) -> Invocation<'_> {
        if self.depth.get() == 0 {
            self.steps.set(0);
        }
        self.depth.set(self.depth.get() + 1);
        Invocation(&self.depth)
    }

    pub(crate) fn lua(&self) -> &Lua {
        &self.lua
    }
}

pub(crate) struct Invocation<'a>(&'a Cell<usize>);

impl Drop for Invocation<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}
