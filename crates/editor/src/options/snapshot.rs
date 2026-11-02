use std::collections::HashMap;

use super::{Access, Change, Error, Name, Scope, Value};
use crate::workspace::Workspace;

#[derive(Clone)]
pub struct Snapshot {
    values: HashMap<Name, (Value, Value)>,
}

impl Snapshot {
    pub fn capture(workspace: &Workspace) -> Self {
        Self {
            values: Name::ALL
                .into_iter()
                .map(|name| {
                    (
                        name,
                        (
                            workspace.option(name, Access::Local),
                            workspace.option(name, Access::Global),
                        ),
                    )
                })
                .collect(),
        }
    }

    pub fn get(&self, name: Name, access: Access) -> Value {
        let (local, global) = self.values[&name];
        if access == Access::Global || name.scope() == Scope::Global {
            global
        } else {
            local
        }
    }

    pub fn apply(&mut self, access: Access, change: &Change) -> Result<(), Error> {
        let Some(value) = change.value(
            self.get(change.name, access),
            self.get(change.name, Access::Global),
        )?
        else {
            return Ok(());
        };
        if let Some((local, global)) = self.values.get_mut(&change.name) {
            if access != Access::Global || change.name.scope() == Scope::Global {
                *local = value;
            }
            if access != Access::Local || change.name.scope() == Scope::Global {
                *global = value;
            }
        }
        Ok(())
    }
}

impl Default for Snapshot {
    fn default() -> Self {
        Self {
            values: Name::ALL
                .into_iter()
                .map(|name| (name, (name.default_value(), name.default_value())))
                .collect(),
        }
    }
}
