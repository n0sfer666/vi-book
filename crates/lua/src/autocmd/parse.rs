use mlua::{Table, Value};
use v_ide_editor::{
    autocmd::{Bus, EventName, GroupId},
    execution::Context,
    workspace::DocId,
};

use crate::options::failure;

pub(super) fn keys(table: &Table, allowed: &[&str]) -> mlua::Result<()> {
    for pair in table.pairs::<Value, Value>() {
        let (key, _) = pair?;
        let name = text(key)?;
        if !allowed.contains(&name.as_str()) {
            return Err(failure(format!("invalid key: {name}")));
        }
    }
    Ok(())
}

pub(super) fn table(value: Value) -> mlua::Result<Table> {
    match value {
        Value::Table(table) => Ok(table),
        _ => Err(failure("expected options table")),
    }
}

pub(super) fn text(value: Value) -> mlua::Result<String> {
    match value {
        Value::String(text) => Ok(text.to_str()?.to_string()),
        _ => Err(failure("expected string")),
    }
}

pub(super) fn integer(value: Value) -> mlua::Result<i64> {
    match value {
        Value::Integer(value) => Ok(value),
        Value::Number(value) if value.is_finite() && value.fract() == 0.0 => {
            value.to_string().parse().map_err(failure)
        }
        _ => Err(failure("expected integer")),
    }
}

pub(super) fn unsigned(value: Value) -> mlua::Result<u64> {
    u64::try_from(integer(value)?).map_err(failure)
}

pub(super) fn boolean(options: &Table, key: &str, default: bool) -> mlua::Result<bool> {
    match options.raw_get::<Value>(key)? {
        Value::Nil => Ok(default),
        Value::Boolean(value) => Ok(value),
        Value::Integer(value) => Ok(value != 0),
        Value::Number(value) => Ok(value != 0.0),
        _ => Err(failure(format!("{key} must be boolean"))),
    }
}

pub(super) fn list(value: Value, codec: &crate::data::Codec) -> mlua::Result<Vec<Value>> {
    let Value::Table(table) = value else {
        return Ok(vec![value]);
    };
    if codec.is_empty_dictionary(&table) {
        return Err(failure("expected array, got dictionary"));
    }
    let mut rows = Vec::new();
    for pair in table.pairs::<Value, Value>() {
        let (key, value) = pair?;
        let index = unsigned(key)?;
        if index == 0 || index > u64::try_from(table.raw_len()).map_err(failure)? {
            return Err(failure("expected array"));
        }
        rows.push((index, value));
    }
    rows.sort_by_key(|(index, _)| *index);
    if rows.len() != table.raw_len() {
        return Err(failure("expected contiguous array"));
    }
    Ok(rows.into_iter().map(|(_, value)| value).collect())
}

pub(super) fn strings(value: Value, codec: &crate::data::Codec) -> mlua::Result<Vec<String>> {
    list(value, codec)?.into_iter().map(text).collect()
}

pub(super) fn events(value: Value, codec: &crate::data::Codec) -> mlua::Result<Vec<String>> {
    let names = strings(value, codec)?;
    for name in &names {
        EventName::parse(name).map_err(failure)?;
    }
    Ok(names)
}

pub(super) fn group(bus: &Bus, value: Value) -> mlua::Result<Option<GroupId>> {
    match value {
        Value::Nil => Ok(None),
        Value::String(name) => {
            let name = name.to_str()?;
            bus.group(&name)
                .map(Some)
                .ok_or_else(|| failure(format!("Invalid group: {name}")))
        }
        value => {
            let id = GroupId(unsigned(value)?);
            bus.group_name(id)
                .map(|_| Some(id))
                .ok_or_else(|| failure(format!("Invalid group: {id}")))
        }
    }
}

pub(super) fn buffer(cx: Option<&Context<'_>>, value: Value) -> mlua::Result<DocId> {
    let number = unsigned(value)?;
    cx.ok_or_else(|| failure("buffer addressing requires a live context"))?
        .buffer(number)
        .ok_or_else(|| failure(format!("Invalid buffer id: {number}")))
}

pub(super) fn selector(
    cx: Option<&Context<'_>>,
    active_buffer: Option<DocId>,
    source: &str,
) -> Result<v_ide_editor::autocmd::Selector, v_ide_editor::autocmd::Error> {
    use v_ide_editor::autocmd::{Error, Selector};
    if source == "<buffer=abuf>"
        && let Some(buffer) = active_buffer
    {
        return Ok(Selector::Buffer(buffer));
    }
    let number = if source == "<buffer>" {
        0
    } else if let Some(number) = source
        .strip_prefix("<buffer=")
        .and_then(|text| text.strip_suffix('>'))
    {
        number.parse::<u64>().unwrap_or(0)
    } else {
        return Ok(Selector::File(source.into()));
    };
    cx.and_then(|cx| cx.buffer(number))
        .map(Selector::Buffer)
        .ok_or(Error::Buffer(number))
}
