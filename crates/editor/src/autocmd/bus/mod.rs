mod definitions;
mod dispatch;
mod edit;
mod ex_groups;

use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::execution::OptionSet;
use crate::workspace::DocId;

use super::{Action, GroupId, Pattern, RecordId, Registry};

pub struct Bus {
    state: RefCell<State>,
    frames: RefCell<Vec<Frame>>,
    depth: Cell<usize>,
    ignore_case: Cell<bool>,
    current_group: Cell<Option<GroupId>>,
}

struct State {
    registry: Registry<Action>,
    compiled: HashMap<RecordId, Rc<Pattern>>,
    retired: HashSet<GroupId>,
}

struct Frame {
    nested: bool,
    option: Option<OptionSet>,
    buffer: Option<DocId>,
}

impl Bus {
    pub fn new(ignore_case: bool) -> Self {
        Self {
            state: RefCell::new(State {
                registry: Registry::new(),
                compiled: HashMap::new(),
                retired: HashSet::new(),
            }),
            frames: RefCell::new(Vec::new()),
            depth: Cell::new(0),
            ignore_case: Cell::new(ignore_case),
            current_group: Cell::new(None),
        }
    }

    pub fn set_ignore_case(&self, ignore_case: bool) {
        self.ignore_case.set(ignore_case);
    }

    pub fn option_values(&self) -> Option<OptionSet> {
        self.frames
            .borrow()
            .iter()
            .rev()
            .find_map(|frame| frame.option.clone())
    }

    pub fn event_buffer(&self) -> Option<DocId> {
        self.frames.borrow().last().and_then(|frame| frame.buffer)
    }
}
