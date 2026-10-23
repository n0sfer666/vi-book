use crate::index::Index;
use crate::names::Names;

pub enum Level<'a> {
    Facts(&'a mut Index),
    Names(&'a mut Names),
}
