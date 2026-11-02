use std::rc::Rc;

use mlua::{Table, Value};
use v_ide_editor::{
    autocmd::{Action, Attributes, Selector},
    execution::Context,
};

use super::{State, callback::LuaCallback, parse};
use crate::options::failure;

impl State {
    pub(super) fn create(
        self: &Rc<Self>,
        cx: Option<&Context<'_>>,
        events: Value,
        options: Table,
    ) -> mlua::Result<u64> {
        parse::keys(
            &options,
            &[
                "group", "pattern", "buffer", "callback", "command", "once", "nested", "desc",
            ],
        )?;
        let events = parse::strings(events, &self.codec)?;
        if events.is_empty() {
            return Err(failure("Required: event"));
        }
        let bus = self.bus()?;
        let attributes = Attributes {
            group: parse::group(&bus, options.raw_get("group")?)?,
            once: parse::boolean(&options, "once", false)?,
            nested: parse::boolean(&options, "nested", false)?,
            description: match options.raw_get::<Value>("desc")? {
                Value::Nil => None,
                value => Some(parse::text(value)?),
            },
        };
        let buffer = options.raw_get::<Value>("buffer")?;
        let pattern = options.raw_get::<Value>("pattern")?;
        let selectors = if !buffer.is_nil() {
            if !pattern.is_nil() {
                return Err(failure("cannot use both pattern and buffer"));
            }
            vec![Ok(Selector::Buffer(parse::buffer(cx, buffer)?))]
        } else if pattern.is_nil() {
            vec![Ok(Selector::File("*".into()))]
        } else {
            let sources = parse::strings(pattern, &self.codec)?;
            let patterns: Vec<_> = sources
                .iter()
                .flat_map(|source| v_ide_editor::autocmd::patterns(source))
                .collect();
            if patterns.is_empty() {
                vec![Ok(Selector::File("*".into()))]
            } else {
                patterns
                    .into_iter()
                    .map(|source| super::parse::selector(cx, bus.event_buffer(), source))
                    .collect()
            }
        };
        let command = options.raw_get::<Value>("command")?;
        let callback = options.raw_get::<Value>("callback")?;
        let action = match (command, callback) {
            (Value::Nil, Value::Function(function)) => {
                Action::callback(LuaCallback::direct(self.clone(), function))
            }
            (Value::Nil, Value::String(name)) => {
                Action::callback(LuaCallback::named(self.clone(), name.to_str()?.to_string()))
            }
            (Value::String(command), Value::Nil) => {
                Action::Command(Rc::from(command.to_str()?.as_ref()))
            }
            _ => return Err(failure("Required: exactly one of command or callback")),
        };
        let events: Vec<_> = events.iter().map(String::as_str).collect();
        bus.define_checked(attributes, action, &events, &selectors)
            .map(|id| id.0)
            .map_err(failure)
    }
}
