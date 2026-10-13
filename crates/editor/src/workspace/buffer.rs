use crate::document::Document;
use crate::workspace::DocId;

pub struct Buffer {
    pub id: DocId,
    pub doc: Document,
}
