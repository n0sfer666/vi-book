use mlua::{Table, Value};
use v_ide_editor::{
    autocmd::{EventName, GroupFilter, Occurrence, Source, patterns},
    execution::Context,
};

use super::{State, parse};
use crate::options::failure;

impl State {
    pub(super) fn exec(
        &self,
        cx: Option<&Context<'_>>,
        events: Value,
        options: Table,
    ) -> mlua::Result<()> {
        let cx = cx.ok_or_else(|| failure("exec_autocmds requires a live context"))?;
        parse::keys(
            &options,
            &["group", "pattern", "buffer", "buf", "modeline", "data"],
        )?;
        let events = parse::strings(events, &self.codec)?;
        let bus = self.bus()?;
        let group = match parse::group(&bus, options.raw_get("group")?)? {
            Some(id) => GroupFilter::Group(id),
            None => GroupFilter::All,
        };
        let modeline = parse::boolean(&options, "modeline", true)?;
        let buffer = options.raw_get::<Value>("buffer")?;
        let alias = options.raw_get::<Value>("buf")?;
        if !buffer.is_nil() && !alias.is_nil() {
            return Err(failure("buf not allowed with buffer"));
        }
        let buffer = if buffer.is_nil() { alias } else { buffer };
        let pattern = options.raw_get::<Value>("pattern")?;
        if !buffer.is_nil() && !pattern.is_nil() {
            return Err(failure("cannot use both pattern and buffer"));
        }
        let buffer = if buffer.is_nil() {
            cx.buffer(0).ok_or_else(|| failure("no current buffer"))?
        } else {
            parse::buffer(Some(cx), buffer)?
        };
        let sources = if pattern.is_nil() {
            Vec::new()
        } else {
            parse::strings(pattern, &self.codec)?
        };
        let mut names: Vec<_> = sources
            .iter()
            .flat_map(|source| patterns(source))
            .map(|pattern| Some(pattern.to_string()))
            .collect();
        if names.is_empty() {
            names.push(None);
        }
        let data = match options.raw_get::<Value>("data")? {
            Value::Nil => None,
            value => Some(self.codec.encode(value)?),
        };
        let mut called = 0;
        for event in events {
            let event = EventName::parse(&event).map_err(failure)?;
            for name in &names {
                let names = cx
                    .autocmd_names(event, buffer, name.as_deref())
                    .ok_or_else(|| failure("invalid event buffer"))?;
                let mut occurrence = Occurrence::new(event, Some(buffer), names);
                occurrence.data = data.as_ref().map(|encoded| encoded.data().clone());
                called += bus.dispatch(cx, &occurrence, group, Source::Explicit);
            }
        }
        if modeline && called > 0 {
            cx.process_modelines();
        }
        Ok(())
    }
}
