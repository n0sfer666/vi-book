use crate::document::Document;
use crate::workspace::DocId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Buffer {
    pub id: DocId,
}

pub struct BufferRef<'a> {
    pub id: DocId,
    pub doc: &'a Document,
}
