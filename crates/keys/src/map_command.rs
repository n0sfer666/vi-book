use crate::map_args::{MapArgs, take_word};
use crate::{Key, MapError, MapMode, parse_keys_with_leader};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapRhs {
    Keys(Vec<Key>),
    Cmd(String),
    Sequence(Vec<MapRhs>),
}

impl MapRhs {
    pub fn parse(input: &str, leader: char) -> Result<Self, MapError> {
        Self::parse_with_leader(input, &[Key::char(leader)])
    }

    pub fn parse_with_leader(input: &str, leader: &[Key]) -> Result<Self, MapError> {
        let mut rest = input;
        let mut parts = Vec::new();
        while let Some(at) = rest.to_ascii_lowercase().find("<cmd>") {
            if at > 0 {
                parts.push(Self::keys(&rest[..at], leader));
            }
            rest = &rest[at + "<cmd>".len()..];
            let end = rest
                .to_ascii_lowercase()
                .find("<cr>")
                .ok_or(MapError::MissingCr)?;
            if rest[..end].to_ascii_lowercase().contains("<cmd>") {
                return Err(MapError::MissingCr);
            }
            parts.push(Self::Cmd(rest[..end].to_string()));
            rest = &rest[end + "<cr>".len()..];
        }
        if !rest.is_empty() || parts.is_empty() {
            parts.push(Self::keys(rest, leader));
        }
        if parts.len() == 1 {
            return Ok(parts.remove(0));
        }
        Ok(Self::Sequence(parts))
    }

    fn keys(input: &str, leader: &[Key]) -> Self {
        let keys = if input.eq_ignore_ascii_case("<nop>") {
            Vec::new()
        } else {
            parse_keys_with_leader(input, leader)
        };
        Self::Keys(keys)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapCommand {
    Map {
        modes: Vec<MapMode>,
        lhs: Vec<Key>,
        rhs: MapRhs,
        remap: bool,
        nowait: bool,
        unique: bool,
    },
    Unmap {
        modes: Vec<MapMode>,
        lhs: Vec<Key>,
    },
    Clear {
        modes: Vec<MapMode>,
    },
    List {
        modes: Vec<MapMode>,
        lhs: Vec<Key>,
    },
}

enum Operation {
    Map(bool),
    Unmap,
    Clear,
}

impl MapCommand {
    pub fn parse(input: &str, leader: char) -> Result<Option<Self>, MapError> {
        Self::parse_with_leader(input, &[Key::char(leader)])
    }

    pub fn parse_with_leader(input: &str, leader: &[Key]) -> Result<Option<Self>, MapError> {
        let (name, rest) = take_word(input);
        let Some((modes, operation)) = command(name) else {
            return Ok(None);
        };
        let (args, rest) = MapArgs::parse(rest)?;
        let (lhs, rhs) = take_word(rest);
        let lhs = parse_keys_with_leader(lhs, leader);
        let result = match operation {
            Operation::Clear if lhs.is_empty() && rhs.is_empty() => Self::Clear { modes },
            Operation::Clear => return Err(MapError::InvalidArgument),
            Operation::Unmap if !lhs.is_empty() && rhs.is_empty() => Self::Unmap { modes, lhs },
            Operation::Unmap => return Err(MapError::InvalidArgument),
            Operation::Map(_) if rhs.is_empty() => Self::List { modes, lhs },
            Operation::Map(remap) => Self::Map {
                modes,
                lhs,
                rhs: MapRhs::parse_with_leader(rhs, leader)?,
                remap,
                nowait: args.nowait,
                unique: args.unique,
            },
        };
        Ok(Some(result))
    }
}

fn command(name: &str) -> Option<(Vec<MapMode>, Operation)> {
    let (base, bang) = name.strip_suffix('!').map_or((name, false), |n| (n, true));
    let (prefix, operation) = if let Some(prefix) = base.strip_suffix("noremap") {
        (prefix, Operation::Map(false))
    } else if let Some(prefix) = base.strip_suffix("unmap") {
        (prefix, Operation::Unmap)
    } else if let Some(prefix) = base.strip_suffix("mapclear") {
        (prefix, Operation::Clear)
    } else {
        (base.strip_suffix("map")?, Operation::Map(true))
    };
    let modes = match (prefix, bang) {
        ("", true) => vec![MapMode::Insert, MapMode::Cmdline],
        ("", false) => vec![MapMode::Normal, MapMode::Visual, MapMode::OpPending],
        ("n", false) => vec![MapMode::Normal],
        ("v" | "x", false) => vec![MapMode::Visual],
        ("o", false) => vec![MapMode::OpPending],
        ("i", false) => vec![MapMode::Insert],
        ("c", false) => vec![MapMode::Cmdline],
        _ => return None,
    };
    Some((modes, operation))
}
