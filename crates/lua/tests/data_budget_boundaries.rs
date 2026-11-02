use mlua::{Lua, Value};
use v_ide_editor::autocmd::{DataBuilder, DataNode};
use v_ide_lua::data::Codec;

#[test]
fn one_million_expanded_values_are_allowed_and_the_next_is_rejected() {
    let lua = Lua::new();
    let codec = Codec::new(&lua).unwrap();
    let input: Value = lua.load("return {[999999]=true}").eval().unwrap();
    let encoded = codec.encode(input).unwrap();
    let Value::Table(copy) = codec.decode(encoded.data()).unwrap() else {
        panic!("missing array")
    };
    assert_eq!(copy.raw_len(), 999999);
    assert_eq!(copy.raw_get::<Value>(1).unwrap(), Value::NULL);
    assert!(copy.raw_get::<bool>(999999).unwrap());

    let mut builder = DataBuilder::new();
    let leaf = builder.push(DataNode::Null).unwrap();
    let root = builder
        .push(DataNode::Array(vec![leaf; 1_000_000]))
        .unwrap();
    let data = builder.finish(root).unwrap();
    assert!(
        codec
            .decode(&data)
            .unwrap_err()
            .to_string()
            .contains("data conversion budget")
    );
}

#[test]
fn sixty_four_mebibytes_of_expanded_strings_are_allowed_and_the_next_byte_is_rejected() {
    let lua = Lua::new();
    let codec = Codec::new(&lua).unwrap();
    let input: Value = lua
        .load("local s=string.rep('x',1048576);local t={};for i=1,64 do t[i]=s end;return t")
        .eval()
        .unwrap();
    let encoded = codec.encode(input).unwrap();
    let Value::Table(copy) = codec.decode(encoded.data()).unwrap() else {
        panic!("missing array")
    };
    assert_eq!(copy.raw_len(), 64);
    assert_eq!(
        copy.raw_get::<mlua::String>(64).unwrap().as_bytes().len(),
        1048576
    );

    let input: Value = lua
        .load("local s=string.rep('x',1048576);local t={};for i=1,64 do t[i]=s end;t[65]='x';return t")
        .eval()
        .unwrap();
    assert!(
        codec
            .encode(input)
            .err()
            .unwrap()
            .to_string()
            .contains("data conversion budget")
    );
}

#[test]
fn compact_shared_subtrees_are_limited_by_expanded_values_in_both_directions() {
    let lua = Lua::new();
    let codec = Codec::new(&lua).unwrap();
    let input: Value = lua
        .load("local t=true;for i=1,20 do t={t,t} end;return t")
        .eval()
        .unwrap();
    assert!(
        codec
            .encode(input)
            .err()
            .unwrap()
            .to_string()
            .contains("data conversion budget")
    );

    let mut builder = DataBuilder::new();
    let mut root = builder.push(DataNode::Boolean(true)).unwrap();
    for _ in 0..20 {
        root = builder.push(DataNode::Array(vec![root, root])).unwrap();
    }
    assert!(
        codec
            .decode(&builder.finish(root).unwrap())
            .unwrap_err()
            .to_string()
            .contains("data conversion budget")
    );
}
