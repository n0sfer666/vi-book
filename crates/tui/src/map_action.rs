#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MapAction {
    Command(String),
    Menu,
}

pub(crate) fn rhs(value: v_ide_keys::MapRhs) -> v_ide_keys::Rhs<MapAction> {
    use v_ide_keys::{MapRhs, Rhs};
    match value {
        MapRhs::Keys(keys) => Rhs::Keys(keys),
        MapRhs::Cmd(command) => Rhs::Action(MapAction::Command(command)),
        MapRhs::Sequence(parts) => Rhs::Sequence(parts.into_iter().map(rhs).collect()),
    }
}
