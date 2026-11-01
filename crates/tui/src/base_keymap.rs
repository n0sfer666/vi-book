use v_ide_keys::{Keymap, MapMode, Mapping, Rhs, parse_keys};

use crate::map_action::MapAction;

pub(crate) fn build(insert_escape: Option<&str>) -> Keymap<MapAction> {
    let mut keymap = Keymap::default();
    if let Some(seq) = insert_escape {
        let mapping = Mapping {
            rhs: Rhs::Keys(parse_keys("<Esc>")),
            remap: false,
            nowait: false,
        };
        let _ = keymap.insert(MapMode::Insert, &parse_keys(seq), mapping);
    }
    keymap
}
