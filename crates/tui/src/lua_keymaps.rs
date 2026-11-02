use v_ide_keys::{
    Key, Keymap, MapError, MapMode, MapRhs, Mapping, format_keys, parse_keys_with_leader,
};
use v_ide_lua::config::Keymap as LuaKeymap;

use crate::map_action::MapAction;

pub(crate) fn install(
    keymap: &mut Keymap<MapAction>,
    maps: &[LuaKeymap],
    leader: &[Key],
) -> Option<String> {
    maps.iter().fold(None, |last, map| {
        install_one(keymap, map, leader).err().or(last)
    })
}

fn install_one(
    keymap: &mut Keymap<MapAction>,
    map: &LuaKeymap,
    leader: &[Key],
) -> Result<(), String> {
    for name in &map.mode {
        let mode = map_mode(name)
            .ok_or_else(|| format!("keymap: invalid mode \"{name}\" for \"{}\"", map.lhs))?;
        let lhs = parse_keys_with_leader(&map.lhs, leader);
        let rhs = Mapping {
            rhs: crate::map_action::rhs(
                MapRhs::parse_with_leader(&map.rhs, leader).map_err(|error| error.to_string())?,
            ),
            remap: map.remap,
            nowait: map.nowait,
        };
        if map.unique && keymap.contains(mode, &lhs) {
            return Err(MapError::AlreadyExists(format_keys(&lhs)).to_string());
        }
        keymap
            .insert(mode, &lhs, rhs)
            .map_err(|_| String::from("keymap: empty lhs"))?;
    }
    Ok(())
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
