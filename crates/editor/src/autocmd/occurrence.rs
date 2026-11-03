use crate::{execution::OptionSet, workspace::DocId};

use super::{AutocmdId, Data, EventName, GroupId};

#[derive(Debug, Clone)]
pub struct Names {
    pub file: String,
    pub short: String,
    pub full: String,
}

impl Names {
    pub fn same(name: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            file: name.clone(),
            short: name.clone(),
            full: name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Occurrence {
    pub event: EventName,
    pub buffer: Option<DocId>,
    pub names: Names,
    pub data: Option<Data>,
    pub(super) option: Option<OptionSet>,
}

impl Occurrence {
    pub fn new(event: EventName, buffer: Option<DocId>, names: Names) -> Self {
        Self {
            event,
            buffer,
            names,
            data: None,
            option: None,
        }
    }

    pub fn with_data(mut self, data: Data) -> Self {
        self.data = Some(data);
        self
    }
}

pub struct Invocation<'a> {
    pub id: Option<AutocmdId>,
    pub group: Option<GroupId>,
    pub occurrence: &'a Occurrence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Automatic,
    Explicit,
    Ex,
    AutomaticEx,
}

impl Source {
    pub(super) fn automatic(self) -> bool {
        matches!(self, Self::Automatic | Self::AutomaticEx)
    }
    pub(super) fn stops_on_error(self) -> bool {
        matches!(self, Self::Ex | Self::AutomaticEx)
    }
}
