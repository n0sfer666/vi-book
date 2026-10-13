#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesiredCol {
    Col(usize),
    End,
}

impl Default for DesiredCol {
    fn default() -> Self {
        DesiredCol::Col(0)
    }
}
