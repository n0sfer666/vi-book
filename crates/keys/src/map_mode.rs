#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MapMode {
    Normal,
    Insert,
    Visual,
    OpPending,
    Cmdline,
}
