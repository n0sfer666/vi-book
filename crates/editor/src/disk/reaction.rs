#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reaction {
    Reload,
    Conflict,
    Vanished,
}
