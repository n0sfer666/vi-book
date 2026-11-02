use mlua::{Table, Value};
use v_ide_editor::{execution::Context, options::Value as OptionValue};

use super::PluginHost;
use crate::{error::Error, options};

struct Restore {
    api: Table,
    get: Value,
    set: Value,
}

impl Drop for Restore {
    fn drop(&mut self) {
        let _ = self.api.raw_set("__option_get", self.get.clone());
        let _ = self.api.raw_set("__option_set", self.set.clone());
    }
}

impl PluginHost {
    pub fn load_with_context(&self, cx: &Context<'_>, script: &str) -> Result<(), Error> {
        self.with_context(cx, || self.load(script))
    }

    fn with_context<R>(
        &self,
        cx: &Context<'_>,
        run: impl FnOnce() -> Result<R, Error>,
    ) -> Result<R, Error> {
        if !self.bridge.borrow().options.pending.is_empty() {
            return Err(options::failure(
                "pending option changes must be applied before entering a live context",
            )
            .into());
        }
        let api = self.vibook()?;
        self.set_options(cx.snapshot());
        let result = self.rt.lua().scope(|scope| {
            let restore = Restore {
                get: api.raw_get("__option_get")?,
                set: api.raw_get("__option_set")?,
                api,
            };
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
            let result = run();
            drop(restore);
            Ok(result)
        });
        self.set_options(cx.snapshot());
        result?
    }
}
