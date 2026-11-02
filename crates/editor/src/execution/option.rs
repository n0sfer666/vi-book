use std::cell::Cell;

use crate::options::{Access, Change, Name, Value};

use super::{Context, Error};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionSet {
    pub name: Name,
    pub access: Access,
    pub old: Value,
    pub new: Value,
    pub old_local: Option<Value>,
    pub old_global: Option<Value>,
}

pub trait Observer {
    fn option_set(&self, cx: &Context<'_>, event: &OptionSet) -> Result<(), String>;
}

impl<F> Observer for F
where
    F: Fn(&Context<'_>, &OptionSet) -> Result<(), String>,
{
    fn option_set(&self, cx: &Context<'_>, event: &OptionSet) -> Result<(), String> {
        self(cx, event)
    }
}

struct Active<'a>(&'a Cell<bool>);

impl Drop for Active<'_> {
    fn drop(&mut self) {
        self.0.set(false);
    }
}

impl Context<'_> {
    pub fn set_option(&self, access: Access, change: &Change) -> Result<Option<Value>, Error> {
        let event = {
            let mut workspace = self.workspace.borrow_mut();
            let name = change.name;
            let old = workspace.option(name, access);
            let old_local =
                (access != Access::Global).then(|| workspace.option(name, Access::Local));
            let old_global =
                (access != Access::Local).then(|| workspace.option(name, Access::Global));
            let Some(new) = workspace.apply_option(access, change)? else {
                return Ok(None);
            };
            OptionSet {
                name,
                access,
                old,
                new,
                old_local,
                old_global,
            }
        };
        if let Some(observer) = self.observer
            && !self.option_active.get()
        {
            self.option_active.set(true);
            let _active = Active(&self.option_active);
            if let Err(error) = observer.option_set(self, &event) {
                self.errors.set(self.errors.get().saturating_add(1));
                self.last_error.replace(error.clone());
                self.messages.borrow_mut().push(error);
            }
        }
        Ok(Some(event.new))
    }
}
