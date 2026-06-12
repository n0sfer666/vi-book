use mlua::{Lua, LuaOptions, StdLib};

use crate::lua::error::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Capability {
    Os,
    Io,
}

pub struct Runtime {
    lua: Lua,
}

impl Runtime {
    pub fn new() -> Result<Self, Error> {
        let libs = StdLib::STRING | StdLib::TABLE | StdLib::MATH | StdLib::UTF8;
        let lua = Lua::new_with(libs, LuaOptions::default())?;
        for global in ["dofile", "loadfile", "load", "loadstring", "print", "warn"] {
            lua.globals().set(global, mlua::Value::Nil)?;
        }
        Ok(Self { lua })
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
        self.lua.load(script).exec()?;
        Ok(())
    }

    pub fn eval_bool(&self, expr: &str) -> Result<bool, Error> {
        Ok(self.lua.load(expr).eval()?)
    }

    pub fn eval_string(&self, expr: &str) -> Result<String, Error> {
        Ok(self.lua.load(expr).eval()?)
    }

    pub(crate) fn lua(&self) -> &Lua {
        &self.lua
    }
}
