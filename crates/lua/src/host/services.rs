use crate::options::failure;
use mlua::{Scope, Table, Value};
use v_ide_editor::execution::Context;

pub(super) const BACKENDS: &[&str] = &[
    "__buffer_get",
    "__buffer_set",
    "__buffer_cursor",
    "__command",
];

pub(super) fn install<'scope, 'env>(
    scope: &'scope Scope<'scope, 'env>,
    api: &Table,
    cx: &'env Context<'_>,
) -> mlua::Result<()> {
    api.raw_set(
        "__buffer_get",
        scope.create_function(|_, ()| Ok(cx.text()))?,
    )?;
    api.raw_set(
        "__buffer_set",
        scope.create_function(|_, text: String| {
            cx.set_text(&text);
            Ok(())
        })?,
    )?;
    api.raw_set(
        "__buffer_cursor",
        scope.create_function(|_, ()| Ok(cx.cursor()))?,
    )?;
    api.raw_set(
        "__command",
        scope.create_function(|_, input: Value| {
            let Value::String(input) = input else {
                return Err(failure("command must be a string"));
            };
            cx.execute(&input.to_str()?).map_err(|error| match error {
                v_ide_editor::execution::Error::Autocmd(_) => {
                    mlua::Error::external(crate::error::Reported {
                        checkpoint: cx.diagnostic_checkpoint(),
                        error,
                    })
                }
                error => failure(error),
            })
        })?,
    )
}
