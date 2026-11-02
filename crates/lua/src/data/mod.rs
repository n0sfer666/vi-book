mod budget;
mod functions;
mod measure;
mod read;
mod rows;
mod write;

use std::rc::Rc;

use mlua::{Function, Lua, Table, Value, WeakLua};
use v_ide_editor::autocmd::{Data, FunctionRef};

pub(crate) use budget::{Budget, Invocation};
use functions::Functions;
pub(crate) use functions::StoredFunction;

#[derive(Clone)]
pub struct Codec {
    lua: WeakLua,
    empty_dict: Table,
    functions: Rc<Functions>,
    budget: Rc<Budget>,
}

pub struct Encoded {
    data: Data,
    _functions: Vec<Rc<StoredFunction>>,
}

impl Encoded {
    pub fn data(&self) -> &Data {
        &self.data
    }
}

impl Codec {
    pub fn new(lua: &Lua) -> mlua::Result<Self> {
        let empty_dict = lua.create_table()?;
        empty_dict.raw_set(
            "__tostring",
            lua.create_function(|_, ()| Ok("vibook.empty_dict()"))?,
        )?;
        Ok(Self {
            lua: lua.weak(),
            empty_dict,
            functions: Rc::new(Functions::new()),
            budget: Budget::for_lua(lua),
        })
    }

    pub fn install(&self, api: &Table) -> mlua::Result<()> {
        let lua = self.lua()?;
        api.raw_set("NIL", Value::NULL)?;
        let marker = self.empty_dict.clone();
        api.raw_set(
            "empty_dict",
            lua.create_function(move |lua, ()| {
                let table = lua.create_table()?;
                table.set_metatable(Some(marker.clone()))?;
                Ok(table)
            })?,
        )
    }

    pub(crate) fn keep_function(&self, function: Function) -> Rc<StoredFunction> {
        self.functions.keep(function)
    }

    pub(crate) fn is_empty_dictionary(&self, table: &Table) -> bool {
        table.is_empty()
            && table
                .metatable()
                .is_some_and(|marker| marker.to_pointer() == self.empty_dict.to_pointer())
    }

    pub(crate) fn function(&self, reference: FunctionRef) -> mlua::Result<Function> {
        let stored = self.functions.get(reference)?;
        Ok(stored.value.clone())
    }

    fn lua(&self) -> mlua::Result<Lua> {
        self.lua
            .try_upgrade()
            .ok_or_else(|| error("Lua instance is destroyed"))
    }
}

fn error(message: &str) -> mlua::Error {
    mlua::Error::RuntimeError(format!("Invalid 'data': {message}"))
}
