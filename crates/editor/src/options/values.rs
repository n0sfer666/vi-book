use std::collections::HashMap;

use super::{Name, Scope, Value};

#[derive(Clone, Default)]
pub struct Values {
    values: HashMap<Name, Value>,
}

impl Values {
    pub fn get(&self, name: Name) -> Value {
        self.values
            .get(&name)
            .copied()
            .unwrap_or_else(|| name.default_value())
    }

    pub(crate) fn set(&mut self, name: Name, value: Value) {
        self.values.insert(name, value);
    }

    pub(crate) fn scoped(&self, scope: Scope) -> Self {
        Self {
            values: Name::ALL
                .into_iter()
                .filter(|name| name.scope() == scope)
                .map(|name| (name, self.get(name)))
                .collect(),
        }
    }
}
