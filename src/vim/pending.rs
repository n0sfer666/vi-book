use crate::vim::operator::Operator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Stage {
    #[default]
    Start,
    Register,
    G,
    Operator(Operator),
    OperatorG(Operator),
    OperatorObject(Operator, bool),
}

#[derive(Default)]
pub struct Pending {
    pub count: Option<usize>,
    pub op_count: Option<usize>,
    pub register: Option<char>,
    pub stage: Stage,
}

impl Pending {
    pub fn reset(&mut self) {
        *self = Pending::default();
    }

    pub fn total_count(&self) -> usize {
        self.count.unwrap_or(1) * self.op_count.unwrap_or(1)
    }

    pub fn has_count(&self) -> bool {
        self.count.is_some() || self.op_count.is_some()
    }

    pub fn push_digit(slot: &mut Option<usize>, digit: usize) {
        *slot = Some(slot.unwrap_or(0) * 10 + digit);
    }
}
