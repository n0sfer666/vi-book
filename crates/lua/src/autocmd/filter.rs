use mlua::{Table, Value};
use v_ide_editor::{
    autocmd::{AutocmdId, EventName, Filter, GroupFilter, Selector},
    execution::Context,
};

use super::{State, parse};

impl State {
    pub(super) fn filter(
        &self,
        cx: Option<&Context<'_>>,
        options: &Table,
        clear: bool,
    ) -> mlua::Result<Filter> {
        parse::keys(
            options,
            if clear {
                &["group", "event", "pattern", "buffer"]
            } else {
                &["id", "group", "event", "pattern", "buffer"]
            },
        )?;
        let bus = self.bus()?;
        let group = match parse::group(&bus, options.raw_get("group")?)? {
            Some(id) => GroupFilter::Group(id),
            None if clear => GroupFilter::Default,
            None => GroupFilter::All,
        };
        let id = match options.raw_get::<Value>("id")? {
            Value::Nil => None,
            value if parse::integer(value.clone())? == -1 => None,
            value => Some(AutocmdId(
                u64::try_from(parse::integer(value)?).unwrap_or(0),
            )),
        };
        let events = match options.raw_get::<Value>("event")? {
            Value::Nil => None,
            value => Some(
                parse::events(value, &self.codec)?
                    .iter()
                    .map(|name| EventName::parse(name).map_err(crate::options::failure))
                    .collect::<mlua::Result<Vec<_>>>()?,
            ),
        };
        let buffer = options.raw_get::<Value>("buffer")?;
        let pattern = options.raw_get::<Value>("pattern")?;
        let selectors = if !buffer.is_nil() {
            if !pattern.is_nil() {
                return Err(crate::options::failure(
                    "cannot use both pattern and buffer",
                ));
            }
            let buffers = parse::list(buffer, &self.codec)?;
            if buffers.is_empty() {
                None
            } else {
                Some(
                    buffers
                        .into_iter()
                        .map(|value| parse::buffer(cx, value).map(Selector::Buffer))
                        .collect::<mlua::Result<Vec<_>>>()?,
                )
            }
        } else if pattern.is_nil() {
            None
        } else {
            let patterns = parse::strings(pattern, &self.codec)?;
            if patterns.is_empty() {
                None
            } else {
                Some(
                    patterns
                        .into_iter()
                        .map(|source| {
                            parse::selector(cx, bus.event_buffer(), &source)
                                .unwrap_or(Selector::File(source))
                        })
                        .collect(),
                )
            }
        };
        Ok(Filter {
            id,
            events,
            selectors,
            group,
        })
    }

    pub(super) fn clear(&self, cx: Option<&Context<'_>>, options: Table) -> mlua::Result<()> {
        let filter = self.filter(cx, &options, true)?;
        self.bus()?.clear(&filter);
        Ok(())
    }
}
