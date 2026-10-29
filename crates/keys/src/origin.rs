#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Origin {
    Typed,
    Remap,
    Noremap,
}

impl Origin {
    pub(crate) fn remappable(self) -> bool {
        self != Origin::Noremap
    }
}
