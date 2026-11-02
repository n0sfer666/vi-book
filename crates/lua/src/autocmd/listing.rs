use mlua::{Lua, Table};
use v_ide_editor::{
    autocmd::{Action, Record, Selector},
    execution::Context,
};

use super::State;
use crate::options::failure;

impl State {
    pub(super) fn get(
        &self,
        lua: &Lua,
        cx: Option<&Context<'_>>,
        options: Table,
    ) -> mlua::Result<Table> {
        let filter = self.filter(cx, &options, false)?;
        let bus = self.bus()?;
        let records: Vec<_> = bus
            .entries(&filter)
            .into_iter()
            .map(|record| {
                let group_name = record
                    .attributes
                    .group
                    .and_then(|group| bus.group_name(group));
                let buffer = match &record.selector {
                    Selector::File(_) => None,
                    Selector::Buffer(id) => {
                        Some(cx.and_then(|cx| cx.buffer_number(*id)).ok_or_else(|| {
                            failure("buffer listing requires a live context and valid buffer")
                        })?)
                    }
                };
                Ok((record, group_name, buffer))
            })
            .collect::<mlua::Result<_>>()?;
        let result = lua.create_table_with_capacity(records.len(), 0)?;
        for (index, (record, group_name, buffer)) in records.iter().enumerate() {
            let row = self.row(lua, *buffer, record)?;
            if let Some(group) = record.attributes.group {
                row.raw_set("group", group.0)?;
                row.raw_set("group_name", group_name.as_deref())?;
            }
            result.raw_set(index + 1, row)?;
        }
        Ok(result)
    }

    fn row(&self, lua: &Lua, buffer: Option<u64>, record: &Record<Action>) -> mlua::Result<Table> {
        let row = lua.create_table()?;
        if let Some(id) = record.id {
            row.raw_set("id", id.0)?;
        }
        row.raw_set("event", record.event.as_str())?;
        row.raw_set("once", record.attributes.once)?;
        row.raw_set("desc", record.attributes.description.as_deref())?;
        match &record.selector {
            Selector::File(pattern) => {
                row.raw_set("pattern", pattern.as_str())?;
                row.raw_set("buflocal", false)?;
            }
            Selector::Buffer(_) => {
                let number = buffer.ok_or_else(|| {
                    failure("buffer listing requires a live context and valid buffer")
                })?;
                row.raw_set("pattern", format!("<buffer={number}>"))?;
                row.raw_set("buflocal", true)?;
                row.raw_set("buffer", number)?;
                row.raw_set("buf", number)?;
            }
        }
        row.raw_set("command", record.action.command().unwrap_or(""))?;
        if let Some(reference) = record.action.function() {
            row.raw_set("callback", self.codec.function(reference)?)?;
        }
        if let Action::Callback(callback) = &record.action
            && let Some(name) = callback.name()
        {
            row.raw_set("callback", format!("<vim function: {name}>"))?;
        }
        Ok(row)
    }
}
