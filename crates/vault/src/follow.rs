use crate::index::Resolution;
use crate::link::{self, Link};
use crate::names::Names;
use v_ide_markdown::extract::extract;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Follow {
    Open(String),
    Disambiguate(Vec<String>),
    Create(String),
}

pub fn link_at(source: &str, cursor: usize) -> Option<Link> {
    extract(source)
        .links
        .into_iter()
        .find(|raw| cursor >= raw.start && cursor < raw.end)
        .and_then(|raw| link::parse(&raw.name))
}

pub fn follow(names: &Names, link: &Link) -> Follow {
    match names.resolve(&link.target) {
        Resolution::One(path) => Follow::Open(path),
        Resolution::Many(paths) => Follow::Disambiguate(paths),
        Resolution::None => Follow::Create(link.target.clone()),
    }
}
