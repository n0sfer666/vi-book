#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct WindowId(pub(crate) u64);

impl WindowId {
    pub fn number(self) -> u64 {
        self.0 + 1000
    }
}
