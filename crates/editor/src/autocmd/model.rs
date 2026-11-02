use super::EventName;
use crate::workspace::DocId;

macro_rules! identifier {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub struct $name(pub u64);
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

identifier!(AutocmdId);
identifier!(GroupId);
identifier!(RecordId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Selector {
    File(String),
    Buffer(DocId),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Attributes {
    pub group: Option<GroupId>,
    pub once: bool,
    pub nested: bool,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Record<A> {
    pub key: RecordId,
    pub id: Option<AutocmdId>,
    pub event: EventName,
    pub selector: Selector,
    pub attributes: Attributes,
    pub action: A,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum GroupFilter {
    #[default]
    All,
    Default,
    Group(GroupId),
}

impl GroupFilter {
    pub(super) fn matches(self, group: Option<GroupId>) -> bool {
        match self {
            Self::All => true,
            Self::Default => group.is_none(),
            Self::Group(id) => group == Some(id),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct Filter {
    pub id: Option<AutocmdId>,
    pub events: Option<Vec<EventName>>,
    pub selectors: Option<Vec<Selector>>,
    pub group: GroupFilter,
}

impl Filter {
    pub(super) fn matches<A>(&self, row: &Record<A>) -> bool {
        self.group.matches(row.attributes.group)
            && self.id.is_none_or(|id| row.id == Some(id))
            && self
                .events
                .as_ref()
                .is_none_or(|events| events.contains(&row.event))
            && self
                .selectors
                .as_ref()
                .is_none_or(|selectors| selectors.contains(&row.selector))
    }
}
