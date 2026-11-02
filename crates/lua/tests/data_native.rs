#[path = "fixtures/data_native.rs"]
mod native;

use mlua::{Lua, Value};
use v_ide_lua::data::Codec;

#[test]
fn typed_payload_conversion_matches_neovim_0126_reference_cases() {
    for &(name, setup, check, expected) in native::CASES {
        let lua = Lua::new();
        let codec = Codec::new(&lua).unwrap();
        let api = lua.create_table().unwrap();
        codec.install(&api).unwrap();
        lua.globals().set("vibook", api).unwrap();
        lua.load(setup).exec().unwrap();
        let encoded = codec
            .encode(lua.globals().get::<Value>("input").unwrap())
            .unwrap();
        lua.globals()
            .set("copy", codec.decode(encoded.data()).unwrap())
            .unwrap();
        let actual: String = lua.load(check).eval().unwrap();
        assert_eq!(actual, expected, "{name}");
    }
}
