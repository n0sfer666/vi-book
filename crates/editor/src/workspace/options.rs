use super::Workspace;
use crate::options::{Access, Change, Error, Name, Scope, Value};

impl Workspace {
    pub fn option(&self, name: Name, access: Access) -> Value {
        if name.scope() == Scope::Global {
            return self.options.get(name);
        }
        if access == Access::Global {
            return if name.scope() == Scope::Window {
                self.windows[self.active].defaults.get(name)
            } else {
                self.options.get(name)
            };
        }
        match name.scope() {
            Scope::Global => self.options.get(name),
            Scope::Buffer => self.current().options.get(name),
            Scope::Window => self.windows[self.active].options[&self.current_id()].get(name),
        }
    }

    pub fn apply_option(
        &mut self,
        access: Access,
        change: &Change,
    ) -> Result<Option<Value>, Error> {
        let name = change.name;
        let current = self.option(name, access);
        let global = self.option(name, Access::Global);
        let Some(value) = change.value(current, global)? else {
            return Ok(None);
        };
        if access != Access::Local || name.scope() == Scope::Global {
            if name.scope() == Scope::Window {
                self.windows[self.active].defaults.set(name, value);
            } else {
                self.options.set(name, value);
            }
        }
        if access != Access::Global {
            match name.scope() {
                Scope::Global => {}
                Scope::Buffer => self.current_mut().options.set(name, value),
                Scope::Window => {
                    let id = self.current_id();
                    if let Some(options) = self.windows[self.active].options.get_mut(&id) {
                        options.set(name, value);
                    }
                }
            }
        }
        Ok(Some(value))
    }
}
