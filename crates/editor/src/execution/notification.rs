use super::Context;

pub struct Notification {
    pub buffer: crate::workspace::DocId,
    pub event: crate::module::Event,
}

impl Context<'_> {
    pub fn take_events(&self) -> Vec<Notification> {
        std::mem::take(&mut self.events.borrow_mut())
    }
}
