mod callback;
mod define;
mod filter;
mod listing;
mod parse;

use std::rc::{Rc, Weak};

use mlua::{Function, Lua, MultiValue, Scope, Table, Value};
use v_ide_editor::{autocmd::Bus, execution::Context};

use crate::{config::Shared, data::Codec, options::failure, runtime::Runtime};

pub(crate) struct Native {
    pub state: Rc<State>,
    _bus: Rc<Bus>,
}

pub(crate) struct State {
    rt: Rc<Runtime>,
    bridge: Shared,
    codec: Codec,
    named_call: Function,
    bus: Weak<Bus>,
}

pub(crate) const BACKENDS: &[&str] = &["__autocmd_create", "__autocmd_get", "__autocmd_clear"];

impl Native {
    pub fn new(rt: Rc<Runtime>, bridge: Shared, bus: Rc<Bus>) -> mlua::Result<Self> {
        let codec = Codec::new(rt.lua())?;
        let named_call = rt
            .lua()
            .load("return function(callback) callback() end")
            .eval()?;
        let state = Rc::new(State {
            rt,
            bridge,
            codec,
            named_call,
            bus: Rc::downgrade(&bus),
        });
        let api: Table = state.rt.lua().globals().raw_get("vibook")?;
        state.codec.install(&api)?;
        for (operation, name) in BACKENDS.iter().enumerate() {
            let weak = Rc::downgrade(&state);
            api.raw_set(
                *name,
                state
                    .rt
                    .lua()
                    .create_function(move |lua, args: MultiValue| {
                        let state = weak
                            .upgrade()
                            .ok_or_else(|| failure("autocmd host is destroyed"))?;
                        state.call(lua, operation, None, args)
                    })?,
            )?;
        }
        install_groups(state.rt.lua(), &api, &bus)?;
        state
            .rt
            .exec(include_str!("../../assets/autocmd.lua"))
            .map_err(failure)?;
        Ok(Self { state, _bus: bus })
    }
}

impl State {
    pub fn install_scoped<'scope, 'env>(
        self: &Rc<Self>,
        scope: &'scope Scope<'scope, 'env>,
        api: &Table,
        cx: &'env Context<'_>,
    ) -> mlua::Result<()> {
        for (operation, name) in BACKENDS.iter().enumerate() {
            let state = self.clone();
            api.raw_set(
                *name,
                scope.create_function(move |lua, args: MultiValue| {
                    state.call(lua, operation, Some(cx), args)
                })?,
            )?;
        }
        Ok(())
    }

    fn bus(&self) -> mlua::Result<Rc<Bus>> {
        self.bus
            .upgrade()
            .ok_or_else(|| failure("autocmd bus is destroyed"))
    }

    fn call(
        self: &Rc<Self>,
        lua: &Lua,
        operation: usize,
        cx: Option<&Context<'_>>,
        mut args: MultiValue,
    ) -> mlua::Result<Value> {
        match operation {
            0 => {
                let events = args.pop_front().unwrap_or(Value::Nil);
                let options = parse::table(args.pop_front().unwrap_or(Value::Nil))?;
                Ok(Value::Integer(
                    i64::try_from(self.create(cx, events, options)?).map_err(failure)?,
                ))
            }
            1 => Ok(Value::Table(self.get(
                lua,
                cx,
                parse::table(args.pop_front().unwrap_or(Value::Nil))?,
            )?)),
            _ => {
                self.clear(cx, parse::table(args.pop_front().unwrap_or(Value::Nil))?)?;
                Ok(Value::Nil)
            }
        }
    }
}

fn install_groups(lua: &Lua, api: &Table, bus: &Rc<Bus>) -> mlua::Result<()> {
    let weak = Rc::downgrade(bus);
    api.raw_set(
        "create_augroup",
        lua.create_function(move |_, (name, options): (Value, Table)| {
            let name = parse::text(name)?;
            if name.is_empty() {
                return Err(failure("Invalid augroup name"));
            }
            parse::keys(&options, &["clear"])?;
            let clear = parse::boolean(&options, "clear", true)?;
            let bus = weak
                .upgrade()
                .ok_or_else(|| failure("autocmd bus is destroyed"))?;
            i64::try_from(bus.create_group(&name, clear).0).map_err(failure)
        })?,
    )?;
    let weak = Rc::downgrade(bus);
    api.raw_set(
        "del_augroup_by_id",
        lua.create_function(move |_, id: Value| {
            let id = parse::unsigned(id)?;
            weak.upgrade()
                .ok_or_else(|| failure("autocmd bus is destroyed"))?
                .delete_group(v_ide_editor::autocmd::GroupId(id))
                .map_err(failure)
        })?,
    )?;
    let weak = Rc::downgrade(bus);
    api.raw_set(
        "del_augroup_by_name",
        lua.create_function(move |_, name: Value| {
            let name = parse::text(name)?;
            weak.upgrade()
                .ok_or_else(|| failure("autocmd bus is destroyed"))?
                .delete_group_by_name(&name)
                .map_err(failure)
        })?,
    )?;
    let weak = Rc::downgrade(bus);
    api.raw_set(
        "del_autocmd",
        lua.create_function(move |_, id: Value| {
            let id = parse::unsigned(id)?;
            weak.upgrade()
                .ok_or_else(|| failure("autocmd bus is destroyed"))?
                .delete(v_ide_editor::autocmd::AutocmdId(id))
                .map_err(failure)
        })?,
    )
}

#[cfg(test)]
#[path = "../../tests/unit/autocmd_bridge.rs"]
mod tests;
