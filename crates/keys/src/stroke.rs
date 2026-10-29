use crate::key::Key;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Stroke {
    pub key: Key,
    pub base: Option<Key>,
}
