use std::rc::Rc;

use mlua::{Lua, Table, Value};
use v_ide_editor::{execution::Context, options::Value as OptionValue};

use super::{PluginHost, services};
use crate::{
    autocmd::{self, State},
    config::Shared,
    error::Error,
    options,
    runtime::Runtime,
};

struct Restore {
    api: Table,
    fields: Vec<(&'static str, Value)>,
}

impl Drop for Restore {
    fn drop(&mut self) {
        for (name, value) in &self.fields {
            let _ = self.api.raw_set(*name, value.clone());
        }
    }
}

struct GcPause<'a> {
    lua: &'a Lua,
    restart: bool,
    paused: bool,
}

impl<'a> GcPause<'a> {
    fn new(lua: &'a Lua) -> Self {
        let mut guard = Self {
            lua,
            restart: false,
            paused: false,
        };
        guard.pause();
        guard
    }

    fn pause(&mut self) {
        if !self.paused {
            self.restart = self.lua.gc_is_running();
            self.lua.gc_stop();
            self.paused = true;
        }
    }

    fn resume(&mut self) {
        if self.paused && self.restart {
            self.lua.gc_restart();
        }
        self.paused = false;
    }
}

impl Drop for GcPause<'_> {
    fn drop(&mut self) {
        self.resume();
    }
}

impl PluginHost {
    pub fn call_with_context<T>(
        &self,
        cx: &Context<'_>,
        run: impl FnOnce(&Self) -> Result<T, Error>,
    ) -> Result<T, Error> {
        let _invocation = self.rt.enter();
        with_context(
            &self.rt,
            &self.bridge,
            self.native.as_ref().map(|native| &native.state),
            cx,
            || run(self),
        )
    }

    pub fn load_with_context(&self, cx: &Context<'_>, script: &str) -> Result<(), Error> {
        let _invocation = self.rt.enter();
        let function = self.rt.lua().load(script).into_function()?;
        with_context(
            &self.rt,
            &self.bridge,
            self.native.as_ref().map(|native| &native.state),
            cx,
            || function.call::<()>(()).map_err(Into::into),
        )
    }
}

pub(crate) fn with_context<R>(
    rt: &Runtime,
    bridge: &Shared,
    native: Option<&Rc<State>>,
    cx: &Context<'_>,
    run: impl FnOnce() -> Result<R, Error>,
) -> Result<R, Error> {
    let _invocation = rt.enter();
    if !bridge.borrow().options.pending.is_empty() {
        return Err(options::failure(
            "pending option changes must be applied before entering a live context",
        )
        .into());
    }
    if bridge.borrow().buffer_pending {
        return Err(options::failure(
            "pending buffer changes must be applied before entering a live context",
        )
        .into());
    }
    let lua = rt.lua();
    let mut gc = GcPause::new(lua);
    let api: Table = lua.globals().raw_get("vibook")?;
    snapshot(bridge, cx);
    let result = lua.scope(|scope| {
        let mut restore = Restore {
            api,
            fields: Vec::new(),
        };
        for name in ["__option_get", "__option_set"]
            .into_iter()
            .chain(services::BACKENDS.iter().copied())
            .chain(
                native
                    .into_iter()
                    .flat_map(|_| autocmd::BACKENDS.iter().copied()),
            )
        {
            restore.fields.push((name, restore.api.raw_get(name)?));
        }
        restore.api.raw_set(
            "__option_get",
            scope.create_function(|_, (interface, text): (String, String)| {
                let (name, access) = options::resolve(&interface, &text)?;
                Ok(match cx.option(name, access) {
                    OptionValue::Bool(value) => Value::Boolean(value),
                    OptionValue::Number(value) => Value::Integer(value),
                })
            })?,
        )?;
        restore.api.raw_set(
            "__option_set",
            scope.create_function(|_, (interface, text, value): (String, String, Value)| {
                let (access, change) = options::change(&interface, &text, value)?;
                cx.set_option(access, &change).map_err(options::failure)?;
                Ok(())
            })?,
        )?;
        services::install(scope, &restore.api, cx)?;
        if let Some(native) = native {
            native.install_scoped(scope, &restore.api, cx)?;
        }
        gc.resume();
        let result = run();
        gc.pause();
        drop(restore);
        Ok(result)
    });
    snapshot(bridge, cx);
    drop(gc);
    result?
}

fn snapshot(bridge: &Shared, cx: &Context<'_>) {
    let mut state = bridge.borrow_mut();
    state.options.current = cx.snapshot();
    if !state.buffer_pending {
        state.buf_text = cx.text();
        state.buf_cursor = cx.cursor();
    }
}
