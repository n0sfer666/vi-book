use crate::names::{Names, Resolution};
use v_ide_markdown::link::Link;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Follow {
    Open(String),
    Disambiguate(Vec<String>),
    Create(String),
}

pub fn follow(names: &Names, link: &Link) -> Follow {
    match names.resolve(&link.target) {
        Resolution::One(path) => Follow::Open(path),
        Resolution::Many(paths) => Follow::Disambiguate(paths),
        Resolution::None => Follow::Create(link.target.clone()),
    }
}
