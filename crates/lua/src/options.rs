use mlua::{Table, Value as LuaValue};
use v_ide_editor::options::{Access, Change, Name, Operation, Scope, Snapshot, Value};

use crate::{config::Shared, error::Error, runtime::Runtime};

#[derive(Default)]
pub struct Options {
    pub current: Snapshot,
    pub pending: Vec<(Access, Change)>,
}

pub(crate) fn install(rt: &Runtime, bridge: &Shared) -> Result<(), Error> {
    let lua = rt.lua();
    let api: Table = lua.globals().get("vibook")?;
    let get = bridge.clone();
    api.set(
        "__option_get",
        lua.create_function(move |_, (interface, text): (String, String)| {
            let (name, access) = resolve(&interface, &text)?;
            Ok(match get.borrow().options.current.get(name, access) {
                Value::Bool(value) => LuaValue::Boolean(value),
                Value::Number(value) => LuaValue::Integer(value),
            })
        })?,
    )?;
    let set = bridge.clone();
    api.set(
        "__option_set",
        lua.create_function(
            move |_, (interface, text, value): (String, String, LuaValue)| {
                let (name, access) = resolve(&interface, &text)?;
                let value = convert(value)?;
                let change = Change {
                    name,
                    operation: Operation::Set(value),
                    text: format!(
                        "{}={}",
                        name.as_str(),
                        match value {
                            Value::Bool(value) => value.to_string(),
                            Value::Number(value) => value.to_string(),
                        }
                    ),
                };
                let mut state = set.borrow_mut();
                state
                    .options
                    .current
                    .apply(access, &change)
                    .map_err(failure)?;
                state.options.pending.push((access, change));
                Ok(())
            },
        )?,
    )?;
    rt.exec(include_str!("../assets/options.lua"))?;
    Ok(())
}

fn resolve(interface: &str, text: &str) -> mlua::Result<(Name, Access)> {
    let name = Name::parse(text).ok_or_else(|| failure(format!("Unknown option: {text}")))?;
    let access = match interface {
        "o" | "opt" => Access::Effective,
        "go" | "opt_global" => Access::Global,
        "opt_local" => Access::Local,
        "bo" if name.scope() == Scope::Buffer => Access::Local,
        "wo" if name.scope() == Scope::Window => Access::Effective,
        _ => {
            return Err(failure(format!(
                "invalid scope {interface} for option {text}"
            )));
        }
    };
    Ok((name, access))
}

fn convert(value: LuaValue) -> mlua::Result<Value> {
    match value {
        LuaValue::Boolean(value) => Ok(Value::Bool(value)),
        LuaValue::Integer(value) => Ok(Value::Number(value)),
        LuaValue::Number(value) if value.is_finite() && value.fract() == 0.0 => value
            .to_string()
            .parse::<i64>()
            .map(Value::Number)
            .map_err(|_| failure("invalid numeric option value")),
        _ => Err(failure("invalid option value type")),
    }
}

fn failure(message: impl std::fmt::Display) -> mlua::Error {
    mlua::Error::RuntimeError(message.to_string())
}
