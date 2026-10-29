use crate::key::Key;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Step<A> {
    Key(Key),
    Action(A),
    Wait,
    Idle,
    Recursive,
}
