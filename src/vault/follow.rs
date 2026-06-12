use crate::markdown::extract::extract;
use crate::vault::index::{Index, Resolution};
use crate::vault::link::{self, Link};

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

pub fn follow(index: &Index, link: &Link) -> Follow {
    match index.resolve(&link.target) {
        Resolution::One(path) => Follow::Open(path),
        Resolution::Many(paths) => Follow::Disambiguate(paths),
        Resolution::None => Follow::Create(link.target.clone()),
    }
}
