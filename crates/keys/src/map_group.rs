use crate::{Key, MapMode, Mapping};

pub struct MapGroup<'a, A> {
    pub modes: Vec<MapMode>,
    pub lhs: Vec<Key>,
    pub mapping: &'a Mapping<A>,
}
