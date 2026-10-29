use v_ide_keys::{Key, Keymap, MapMode, Mapping, Rhs, format_keys, parse_keys};
use v_ide_lua::config::Keymap as LuaKeymap;

use crate::map_action::MapAction;

const LEADER: &str = "<leader>";

pub(crate) fn install(
    keymap: &mut Keymap<MapAction>,
    maps: &[LuaKeymap],
    leader: char,
) -> Option<String> {
    maps.iter().fold(None, |last, map| {
        install_one(keymap, map, leader).err().or(last)
    })
}

fn install_one(
    keymap: &mut Keymap<MapAction>,
    map: &LuaKeymap,
    leader: char,
) -> Result<(), String> {
    let mode = map_mode(&map.mode)
        .ok_or_else(|| format!("keymap: invalid mode \"{}\" for \"{}\"", map.mode, map.lhs))?;
    let lhs = parse_keys(&expand_leader(&map.lhs, leader));
    let rhs = mapping(&map.rhs, leader);
    keymap
        .insert(mode, &lhs, rhs)
        .map(drop)
        .map_err(|_| String::from("keymap: empty lhs"))
}

fn map_mode(mode: &str) -> Option<MapMode> {
    match mode {
        "n" => Some(MapMode::Normal),
        "i" => Some(MapMode::Insert),
        "x" | "v" => Some(MapMode::Visual),
        "o" => Some(MapMode::OpPending),
        "c" => Some(MapMode::Cmdline),
        _ => None,
    }
}

fn expand_leader(lhs: &str, leader: char) -> String {
    let leader = format_keys(&[Key::char(leader)]);
    let mut out = String::new();
    let mut rest = lhs;
    while let Some(at) = rest.to_ascii_lowercase().find(LEADER) {
        out.push_str(&rest[..at]);
        out.push_str(&leader);
        rest = &rest[at + LEADER.len()..];
    }
    out.push_str(rest);
    out
}

fn mapping(rhs: &str, leader: char) -> Mapping<MapAction> {
    let rhs = if rhs.eq_ignore_ascii_case("<Nop>") {
        Rhs::Keys(Vec::new())
    } else if let Some(command) = rhs.strip_prefix(':') {
        Rhs::Action(MapAction::Command(command.to_string()))
    } else {
        Rhs::Keys(parse_keys(&expand_leader(rhs, leader)))
    };
    Mapping { rhs, remap: false }
}
