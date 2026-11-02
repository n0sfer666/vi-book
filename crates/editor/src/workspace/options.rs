use super::{DocId, WindowId, Workspace};
use crate::options::{Access, Change, Error, Name, Scope, Value};

impl Workspace {
    pub fn option(&self, name: Name, access: Access) -> Value {
        self.option_at(self.current_id(), self.active_window(), name, access)
            .unwrap_or_else(|| name.default_value())
    }

    pub fn option_at(
        &self,
        doc: DocId,
        window: WindowId,
        name: Name,
        access: Access,
    ) -> Option<Value> {
        if name.scope() == Scope::Global
            || (access == Access::Global && name.scope() == Scope::Buffer)
        {
            return Some(self.options.get(name));
        }
        if name.scope() == Scope::Buffer {
            return Some(self.doc(doc)?.options.get(name));
        }
        let window = self
            .windows
            .iter()
            .find(|candidate| candidate.id == window)?;
        if access == Access::Global {
            Some(window.defaults.get(name))
        } else {
            Some(window.options.get(&doc)?.get(name))
        }
    }

    pub fn apply_option(
        &mut self,
        access: Access,
        change: &Change,
    ) -> Result<Option<Value>, Error> {
        self.apply_option_at(self.current_id(), self.active_window(), access, change)
    }

    pub fn apply_option_at(
        &mut self,
        doc: DocId,
        window: WindowId,
        access: Access,
        change: &Change,
    ) -> Result<Option<Value>, Error> {
        let name = change.name;
        let Some(current) = self.option_at(doc, window, name, access) else {
            return Ok(None);
        };
        let Some(global) = self.option_at(doc, window, name, Access::Global) else {
            return Ok(None);
        };
        let Some(value) = change.value(current, global)? else {
            return Ok(None);
        };
        if access != Access::Local || name.scope() == Scope::Global {
            if name.scope() == Scope::Window {
                if let Some(window) = self
                    .windows
                    .iter_mut()
                    .find(|candidate| candidate.id == window)
                {
                    window.defaults.set(name, value);
                }
            } else {
                self.options.set(name, value);
            }
        }
        if access != Access::Global {
            match name.scope() {
                Scope::Global => {}
                Scope::Buffer => {
                    if let Some(doc) = self.doc_mut(doc) {
                        doc.options.set(name, value);
                    }
                }
                Scope::Window => {
                    if let Some(options) = self
                        .windows
                        .iter_mut()
                        .find(|candidate| candidate.id == window)
                        .and_then(|window| window.options.get_mut(&doc))
                    {
                        options.set(name, value);
                    }
                }
            }
        }
        Ok(Some(value))
    }
}
