use crate::workspace::{Buffer, WindowId};

pub struct Window {
    pub id: WindowId,
    pub buf: Buffer,
}
