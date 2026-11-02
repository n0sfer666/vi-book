use mlua::{Function, Lua, Value};
use v_ide_lua::data::Codec;

#[test]
fn function_identity_is_preserved_and_foreign_handles_require_their_live_owner() {
    let lua = Lua::new();
    let codec = Codec::new(&lua).unwrap();
    let function: Function = lua.load("return function()return 42 end").eval().unwrap();
    lua.globals().set("original", function.clone()).unwrap();
    let encoded = codec.encode(Value::Function(function)).unwrap();
    let decoded = codec.decode(encoded.data()).unwrap();
    lua.globals().set("copy", decoded.clone()).unwrap();
    lua.load("assert(copy==original and copy()==42)")
        .exec()
        .unwrap();
    let data = encoded.data().clone();
    let other_lua = Lua::new();
    let other = Codec::new(&other_lua).unwrap();
    assert!(other.decode(&data).is_err());
    drop(encoded);
    assert!(codec.decode(&data).is_err());
    let Value::Function(retained) = decoded else {
        panic!("missing function")
    };
    assert_eq!(retained.call::<i64>(()).unwrap(), 42);
}

#[test]
fn the_weak_function_pool_does_not_keep_captured_lua_values_alive_after_payload_drop() {
    let lua = Lua::new();
    let codec = Codec::new(&lua).unwrap();
    let function: Function = lua
        .load(
            r#"
        collected=false
        local token=setmetatable({},{__gc=function()collected=true end})
        return function()return token end
    "#,
        )
        .eval()
        .unwrap();
    let encoded = codec.encode(Value::Function(function)).unwrap();
    lua.gc_collect().unwrap();
    assert!(!lua.globals().get::<bool>("collected").unwrap());
    drop(encoded);
    lua.gc_collect().unwrap();
    assert!(lua.globals().get::<bool>("collected").unwrap());
}
