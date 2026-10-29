#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FKey(u8);

impl FKey {
    pub fn new(n: u8) -> Option<FKey> {
        (1..=12).contains(&n).then_some(FKey(n))
    }

    pub fn n(self) -> u8 {
        self.0
    }
}
