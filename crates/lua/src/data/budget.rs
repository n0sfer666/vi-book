use std::{cell::Cell, rc::Rc};

use mlua::Lua;

use super::error;

#[derive(Clone, Copy, Default)]
pub(super) struct Cost {
    pub values: u64,
    pub bytes: u64,
}

impl Cost {
    pub const VALUE: Self = Self {
        values: 1,
        bytes: 0,
    };

    pub fn values(count: usize) -> Self {
        Self {
            values: u64::try_from(count).unwrap_or(u64::MAX),
            bytes: 0,
        }
    }

    pub fn bytes(count: usize) -> Self {
        Self {
            values: 0,
            bytes: u64::try_from(count).unwrap_or(u64::MAX),
        }
    }

    pub fn add(self, other: Self) -> Self {
        Self {
            values: self.values.saturating_add(other.values),
            bytes: self.bytes.saturating_add(other.bytes),
        }
    }
}

pub(crate) struct Budget {
    depth: Cell<usize>,
    used: Cell<Cost>,
    exhausted: Cell<bool>,
}

impl Budget {
    pub(crate) fn for_lua(lua: &Lua) -> Rc<Self> {
        if let Some(budget) = lua.app_data_ref::<Rc<Self>>() {
            return budget.clone();
        }
        let budget = Rc::new(Self {
            depth: Cell::new(0),
            used: Cell::new(Cost::default()),
            exhausted: Cell::new(false),
        });
        lua.set_app_data(budget.clone());
        budget
    }

    pub(crate) fn enter(self: &Rc<Self>) -> Invocation {
        if self.depth.get() == 0 {
            self.used.set(Cost::default());
            self.exhausted.set(false);
        }
        self.depth.set(self.depth.get() + 1);
        Invocation(self.clone())
    }

    pub(super) fn ensure(&self, cost: Cost) -> mlua::Result<()> {
        let total = self.used.get().add(cost);
        if self.exhausted.get() || total.values > 1_000_000 || total.bytes > 64 * 1024 * 1024 {
            self.exhausted.set(true);
            return Err(error("data conversion budget exceeded"));
        }
        Ok(())
    }

    pub(super) fn charge(&self, cost: Cost) -> mlua::Result<()> {
        self.ensure(cost)?;
        self.used.set(self.used.get().add(cost));
        Ok(())
    }
}

pub(crate) struct Invocation(Rc<Budget>);

impl Drop for Invocation {
    fn drop(&mut self) {
        self.0.depth.set(self.0.depth.get() - 1);
    }
}
