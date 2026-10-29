use crate::key::Key;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rhs<A> {
    Keys(Vec<Key>),
    Action(A),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mapping<A> {
    pub rhs: Rhs<A>,
    pub remap: bool,
}
