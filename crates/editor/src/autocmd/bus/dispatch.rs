use std::cell::{Cell, RefCell};

use crate::{
    autocmd::{EventName, GroupFilter, Invocation, Names, Occurrence, Selector, Source},
    execution::{Context, Observer, OptionSet},
};

use super::{Bus, Frame};
use crate::autocmd::action::Failure;

const MAX_DEPTH: usize = 10;

struct Depth<'a>(&'a Cell<usize>);
impl Drop for Depth<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() - 1);
    }
}
struct Active<'a>(&'a RefCell<Vec<Frame>>);
impl Drop for Active<'_> {
    fn drop(&mut self) {
        self.0.borrow_mut().pop();
    }
}

impl Bus {
    pub fn dispatch(
        &self,
        cx: &Context<'_>,
        occurrence: &Occurrence,
        group: GroupFilter,
        source: Source,
    ) -> usize {
        if source == Source::Automatic
            && self
                .frames
                .borrow()
                .last()
                .is_some_and(|frame| !frame.nested)
        {
            return 0;
        }
        if self.depth.get() >= MAX_DEPTH {
            cx.record_error("E218: Autocommand nesting too deep".into());
            return 0;
        }
        let keys: Vec<_> = {
            let state = self.state.borrow();
            state
                .registry
                .rows
                .iter()
                .filter(|row| {
                    row.event == occurrence.event
                        && group.matches(row.attributes.group)
                        && match &row.selector {
                            Selector::Buffer(id) => occurrence.buffer == Some(*id),
                            Selector::File(_) => {
                                state.compiled.get(&row.key).is_some_and(|pattern| {
                                    pattern.matches(
                                        &occurrence.names.short,
                                        &occurrence.names.full,
                                        self.ignore_case.get(),
                                    )
                                })
                            }
                        }
                })
                .map(|row| row.key)
                .collect()
        };
        if keys.is_empty() {
            return 0;
        }
        self.depth.set(self.depth.get() + 1);
        let _depth = Depth(&self.depth);
        let mut called = 0;
        for key in keys {
            let row = {
                let mut state = self.state.borrow_mut();
                let row = state.registry.begin(key);
                if row.as_ref().is_some_and(|row| row.attributes.once) {
                    state.compiled.remove(&key);
                }
                row
            };
            let Some(row) = row else { continue };
            self.frames.borrow_mut().push(Frame {
                nested: row.attributes.nested,
                option: occurrence.option.clone(),
            });
            let _active = Active(&self.frames);
            called += 1;
            let invocation = Invocation {
                id: row.id,
                group: row.attributes.group,
                occurrence,
            };
            match row.action.run(cx, &invocation) {
                Ok(true) => {
                    let mut state = self.state.borrow_mut();
                    state.registry.remove_record(key);
                    state.compiled.remove(&key);
                }
                Ok(false) => {}
                Err(Failure::Reported) => {}
                Err(Failure::Message(error)) => cx.record_error(error),
            }
        }
        called
    }
}

impl Observer for Bus {
    fn option_set(&self, cx: &Context<'_>, option: &OptionSet) -> Result<(), String> {
        let event = EventName::parse("OptionSet").map_err(|error| error.to_string())?;
        let name = option.name.as_str().to_string();
        let mut occurrence = Occurrence::new(
            event,
            None,
            Names {
                file: String::new(),
                short: name.clone(),
                full: name,
            },
        );
        occurrence.option = Some(option.clone());
        self.dispatch(cx, &occurrence, GroupFilter::All, Source::Automatic);
        Ok(())
    }
}
