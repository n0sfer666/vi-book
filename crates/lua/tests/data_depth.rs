use mlua::{Lua, Value};
use v_ide_lua::data::Codec;

#[test]
fn deeply_nested_lua_data_roundtrips_without_recursive_conversion_or_drop() {
    let lua = Lua::new();
    let codec = Codec::new(&lua).unwrap();
    let input: Value = lua
        .load("local t='leaf'; for i=1,20000 do t={t}end; return t")
        .eval()
        .unwrap();
    let encoded = codec.encode(input).unwrap();
    lua.globals()
        .set("copy", codec.decode(encoded.data()).unwrap())
        .unwrap();
    lua.load("local t=copy;for i=1,20000 do t=t[1]end;assert(t=='leaf');copy=nil")
        .exec()
        .unwrap();
    drop(encoded);
    lua.gc_collect().unwrap();
}
