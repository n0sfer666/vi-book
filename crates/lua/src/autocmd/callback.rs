use std::rc::Rc;

use mlua::{Function, Value};
use v_ide_editor::{
    autocmd::{Callback, FunctionRef, Invocation},
    execution::Context,
};

use super::State;
use crate::{data::StoredFunction, host::scoped, options::failure};

pub(super) struct LuaCallback {
    state: Rc<State>,
    target: Target,
}

enum Target {
    Direct(Rc<StoredFunction>),
    Named(String),
}

impl LuaCallback {
    pub fn direct(state: Rc<State>, function: Function) -> Self {
        let target = Target::Direct(state.codec.keep_function(function));
        Self { state, target }
    }

    pub fn named(state: Rc<State>, name: String) -> Self {
        Self {
            state,
            target: Target::Named(name),
        }
    }

    fn invoke(&self, cx: &Context<'_>, invocation: &Invocation<'_>) -> mlua::Result<bool> {
        let lua = self.state.rt.lua();
        let function = match &self.target {
            Target::Direct(stored) => stored.value.clone(),
            Target::Named(name) => {
                let path = name
                    .strip_prefix("v:lua.")
                    .ok_or_else(|| failure(format!("E117: Unknown function: {name}")))?;
                let mut value = Value::Table(lua.globals());
                for part in path.split('.') {
                    value = match value {
                        Value::Table(table) => table.get(part)?,
                        _ => return Err(failure(format!("E117: Unknown function: {name}"))),
                    };
                }
                self.state.named_call.call::<()>(value)?;
                return Ok(false);
            }
        };
        let occurrence = invocation.occurrence;
        let arg = lua.create_table()?;
        arg.raw_set("id", invocation.id.map(|id| id.0))?;
        arg.raw_set("group", invocation.group.map(|id| id.0))?;
        arg.raw_set("event", occurrence.event.as_str())?;
        arg.raw_set(
            "match",
            if occurrence.event.is_file_pattern() {
                occurrence.names.full.as_str()
            } else {
                occurrence.names.short.as_str()
            },
        )?;
        arg.raw_set("file", occurrence.names.file.as_str())?;
        arg.raw_set(
            "buf",
            occurrence
                .buffer
                .and_then(|id| cx.buffer_number(id))
                .unwrap_or(0),
        )?;
        if let Some(data) = &occurrence.data {
            arg.raw_set("data", self.state.codec.decode(data)?)?;
        }
        let result: Value = function.call(arg)?;
        Ok(!matches!(result, Value::Nil | Value::Boolean(false)))
    }
}

impl LuaCallback {
    fn call(
        &self,
        cx: &Context<'_>,
        invocation: &Invocation<'_>,
    ) -> Result<bool, crate::error::Error> {
        scoped::with_context(
            &self.state.rt,
            &self.state.bridge,
            Some(&self.state),
            cx,
            || self.invoke(cx, invocation).map_err(Into::into),
        )
    }
}
impl Callback for LuaCallback {
    fn run(&self, cx: &Context<'_>, invocation: &Invocation<'_>) -> Result<bool, String> {
        self.call(cx, invocation).map_err(|error| error.to_string())
    }
    fn run_checked(
        &self,
        cx: &Context<'_>,
        invocation: &Invocation<'_>,
    ) -> Result<bool, v_ide_editor::autocmd::Failure> {
        self.call(cx, invocation).map_err(|error| {
            if error.is_reported() {
                v_ide_editor::autocmd::Failure::Reported
            } else {
                v_ide_editor::autocmd::Failure::Message(error.to_string())
            }
        })
    }

    fn function(&self) -> Option<FunctionRef> {
        match &self.target {
            Target::Direct(stored) => Some(stored.reference),
            Target::Named(_) => None,
        }
    }

    fn name(&self) -> Option<&str> {
        match &self.target {
            Target::Named(name) => Some(name),
            Target::Direct(_) => None,
        }
    }
}
