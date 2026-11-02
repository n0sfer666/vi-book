use mlua::{Lua, Value};
use v_ide_editor::autocmd::{DataBuilder, DataNode};
use v_ide_lua::data::Codec;

#[test]
fn sparse_array_holes_count_toward_the_value_limit_before_expansion() {
    let lua = Lua::new();
    let codec = Codec::new(&lua).unwrap();
    let input: Value = lua.load("return {[1000000]=true}").eval().unwrap();
    let result = codec.encode(input);
    assert!(result.is_err(), "sparse array must exceed the value budget");
    assert!(
        result
            .err()
            .unwrap()
            .to_string()
            .contains("data conversion budget")
    );
    assert!(codec.encode(Value::Boolean(true)).is_ok());
}

#[test]
fn repeated_table_subtrees_count_their_expanded_string_and_key_bytes() {
    let lua = Lua::new();
    let codec = Codec::new(&lua).unwrap();
    for script in [
        "local s=string.rep('x',1048576);return {s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s,s}",
        "local t={[string.rep('k',1048576)]=true};for i=1,7 do t={t,t}end;return t",
    ] {
        let input: Value = lua.load(script).eval().unwrap();
        let result = codec.encode(input);
        assert!(result.is_err(), "expanded bytes must exceed the budget");
        assert!(
            result
                .err()
                .unwrap()
                .to_string()
                .contains("data conversion budget")
        );
    }
}

#[test]
fn expanded_core_data_is_checked_before_lua_allocation() {
    let lua = Lua::new();
    let codec = Codec::new(&lua).unwrap();
    let mut builder = DataBuilder::new();
    let leaf = builder.push(DataNode::String(vec![b'x'; 1048576])).unwrap();
    let root = builder.push(DataNode::Array(vec![leaf; 65])).unwrap();
    let data = builder.finish(root).unwrap();
    let result = codec.decode(&data);
    assert!(
        result.is_err(),
        "exported copies must exceed the byte budget"
    );
    assert!(
        result
            .err()
            .unwrap()
            .to_string()
            .contains("data conversion budget")
    );
    assert!(codec.encode(Value::Boolean(true)).is_ok());
}
