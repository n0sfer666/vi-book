use crate::{Key, MapError, format_keys};

#[derive(Default)]
pub(crate) struct MapArgs {
    pub(crate) nowait: bool,
    pub(crate) unique: bool,
}

impl MapArgs {
    pub(crate) fn parse(mut rest: &str) -> Result<(Self, &str), MapError> {
        let mut args = Self::default();
        loop {
            let trimmed = rest.trim_start_matches([' ', '\t']);
            let Some(end) = trimmed.find('>') else {
                return Ok((args, trimmed));
            };
            let flag = &trimmed[..=end];
            match flag {
                "<silent>" | "<special>" => {}
                "<nowait>" => args.nowait = true,
                "<unique>" => args.unique = true,
                "<buffer>" | "<expr>" | "<script>" => {
                    return Err(MapError::Unsupported(flag.into()));
                }
                _ => return Ok((args, trimmed)),
            }
            rest = &trimmed[end + 1..];
        }
    }
}

pub(crate) fn take_word(input: &str) -> (&str, &str) {
    let input = input.trim_start_matches([' ', '\t']);
    let end = input.find([' ', '\t']).unwrap_or(input.len());
    (&input[..end], input[end..].trim_start_matches([' ', '\t']))
}

pub(crate) fn expand_leader(input: &str, leader: char) -> String {
    let mut rest = input;
    let mut out = String::new();
    let leader = format_keys(&[Key::char(leader)]);
    while let Some(at) = rest.to_ascii_lowercase().find("<leader>") {
        out.push_str(&rest[..at]);
        out.push_str(&leader);
        rest = &rest[at + "<leader>".len()..];
    }
    out.push_str(rest);
    out
}
