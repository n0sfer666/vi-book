use mlua::{Lua, Value};
use v_ide_lua::data::Codec;

fn codec() -> (Lua, Codec) {
    let lua = Lua::new();
    let codec = Codec::new(&lua).unwrap();
    let api = lua.create_table().unwrap();
    codec.install(&api).unwrap();
    lua.globals().set("vibook", api).unwrap();
    (lua, codec)
}

#[test]
fn typed_data_roundtrips_scalars_binary_strings_nulls_and_dictionary_keys() {
    let (lua, codec) = codec();
    let input: Value = lua
        .load(
            r#"
        input = {flag=true, integer=42, float=1.5, text='a\0\255',
          null=vibook.NIL, array={'x',vibook.NIL,'z'}, ['a\0b']='binary key', ['']='empty key'}
        return input
    "#,
        )
        .eval()
        .unwrap();
    let encoded = codec.encode(input).unwrap();
    lua.globals()
        .set("copy", codec.decode(encoded.data()).unwrap())
        .unwrap();
    lua.load(
        r#"
        assert(copy ~= input and copy.array ~= input.array)
        assert(copy.flag == true and copy.integer == 42 and copy.float == 1.5)
        assert(copy.text == 'a\0\255' and copy['a\0b'] == 'binary key' and copy[''] == 'empty key')
        assert(copy.null == vibook.NIL and copy.array[2] == vibook.NIL)
    "#,
    )
    .exec()
    .unwrap();
    for value in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        let encoded = codec.encode(Value::Number(value)).unwrap();
        let Value::Number(decoded) = codec.decode(encoded.data()).unwrap() else {
            panic!("missing float")
        };
        assert!(decoded == value || (decoded.is_nan() && value.is_nan()));
    }
}

#[test]
fn table_aliases_are_copied_independently_and_metatables_are_not_executed_or_retained() {
    let (lua, codec) = codec();
    let input: Value = lua.load(r#"
        local child={x=1}
        return setmetatable({a=child,b=child},{__pairs=function()error('must not run')end,__index={hidden=true}})
    "#).eval().unwrap();
    let encoded = codec.encode(input).unwrap();
    lua.globals()
        .set("copy", codec.decode(encoded.data()).unwrap())
        .unwrap();
    lua.load("assert(copy.a ~= copy.b and getmetatable(copy) == nil and copy.hidden == nil); copy.a.x=2; assert(copy.b.x==1)").exec().unwrap();
}

#[test]
fn sparse_arrays_get_null_holes_and_empty_array_dictionary_kinds_remain_distinct() {
    let (lua, codec) = codec();
    let input: Value = lua.load("return {array={},dict=vibook.empty_dict(),sparse={[5]='e'},tagged=setmetatable({1,2},getmetatable(vibook.empty_dict())),fake=setmetatable({},{__jsontype='object'})}").eval().unwrap();
    let encoded = codec.encode(input).unwrap();
    lua.globals()
        .set("copy", codec.decode(encoded.data()).unwrap())
        .unwrap();
    lua.load(
        r#"
        assert(getmetatable(copy.array)==nil and getmetatable(copy.fake)==nil)
        assert(getmetatable(copy.dict)==getmetatable(vibook.empty_dict()))
        assert(#copy.sparse==5 and copy.sparse[5]=='e')
        for i=1,4 do assert(copy.sparse[i]==vibook.NIL)end
        assert(getmetatable(copy.tagged)==nil and copy.tagged[1]==1 and copy.tagged[2]==2)
    "#,
    )
    .exec()
    .unwrap();
}

#[test]
fn cycles_mixed_keys_and_unsupported_lua_values_are_rejected() {
    let (lua, codec) = codec();
    for script in [
        "local t={};t.self=t;return t",
        "return {1,key=2}",
        "return {[0]='x'}",
        "return {[-1]='x'}",
        "return {[1.5]='x'}",
        "return {[true]='x'}",
        "return coroutine.create(function()end)",
    ] {
        let input: Value = lua.load(script).eval().unwrap();
        assert!(codec.encode(input).is_err(), "{script}");
    }
}
