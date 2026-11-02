use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::{Rc, Weak},
    sync::atomic::{AtomicU64, Ordering},
};

use mlua::Function;
use v_ide_editor::autocmd::{FunctionId, FunctionRef, RuntimeId};

static NEXT_RUNTIME: AtomicU64 = AtomicU64::new(1);

pub(crate) struct StoredFunction {
    pub reference: FunctionRef,
    pub value: Function,
}

pub(super) struct Functions {
    runtime: RuntimeId,
    next: Cell<u64>,
    sweep_at: Cell<usize>,
    values: RefCell<HashMap<FunctionId, Weak<StoredFunction>>>,
}

impl Functions {
    pub fn new() -> Self {
        Self {
            runtime: RuntimeId(NEXT_RUNTIME.fetch_add(1, Ordering::Relaxed)),
            next: Cell::new(1),
            sweep_at: Cell::new(128),
            values: RefCell::new(HashMap::new()),
        }
    }

    pub fn keep(&self, function: Function) -> Rc<StoredFunction> {
        let reference = FunctionRef {
            runtime: self.runtime,
            function: FunctionId(self.next.get()),
        };
        self.next.set(self.next.get() + 1);
        let stored = Rc::new(StoredFunction {
            reference,
            value: function,
        });
        let mut values = self.values.borrow_mut();
        if values.len() >= self.sweep_at.get() {
            values.retain(|_, value| value.strong_count() > 0);
            self.sweep_at.set(values.len().saturating_mul(2).max(128));
        }
        values.insert(reference.function, Rc::downgrade(&stored));
        stored
    }

    pub fn get(&self, reference: FunctionRef) -> mlua::Result<Rc<StoredFunction>> {
        if reference.runtime != self.runtime {
            return Err(super::error("Function belongs to another Lua runtime"));
        }
        self.values
            .borrow()
            .get(&reference.function)
            .and_then(Weak::upgrade)
            .ok_or_else(|| super::error("Function reference has expired"))
    }
}
