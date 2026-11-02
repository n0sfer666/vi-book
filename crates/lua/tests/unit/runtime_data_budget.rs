use super::Runtime;
use crate::data::Codec;

#[test]
fn separate_codecs_and_nested_runtime_calls_share_the_outer_conversion_budget() {
    let rt = Runtime::new().unwrap();
    let first = Codec::new(rt.lua()).unwrap();
    let second = Codec::new(rt.lua()).unwrap();
    rt.lua()
        .scope(|scope| {
            rt.lua().globals().set(
                "first",
                scope.create_function(|_, value| first.encode(value).map(|_| ()))?,
            )?;
            rt.lua().globals().set(
                "nested",
                scope.create_function(|_, ()| {
                    rt.exec("second({[500000]=true})")
                        .map_err(mlua::Error::external)
                })?,
            )?;
            rt.lua().globals().set(
                "second",
                scope.create_function(|_, value| second.encode(value).map(|_| ()))?,
            )?;
            let result = rt.exec("first({[500000]=true});nested()");
            assert!(result.is_err(), "nested calls must retain the outer spend");
            assert!(
                result
                    .err()
                    .unwrap()
                    .to_string()
                    .contains("data conversion budget")
            );
            rt.exec("first({[500000]=true})").unwrap();
            Ok(())
        })
        .unwrap();
}

#[test]
fn caught_budget_errors_remain_exhausted_until_a_new_outer_call() {
    let rt = Runtime::new().unwrap();
    let codec = Codec::new(rt.lua()).unwrap();
    rt.lua()
        .scope(|scope| {
            rt.lua().globals().set(
                "convert",
                scope.create_function(|_, value| codec.encode(value).map(|_| ()))?,
            )?;
            rt.exec("assert(not pcall(convert,{[1000000]=true})); assert(not pcall(convert,true))")
                .unwrap();
            rt.exec("convert(true)").unwrap();
            Ok(())
        })
        .unwrap();
}

#[test]
fn failed_conversion_keeps_the_cost_of_collected_children_until_the_outer_call_ends() {
    let rt = Runtime::new().unwrap();
    let codec = Codec::new(rt.lua()).unwrap();
    let bad = rt
        .lua()
        .create_thread(rt.lua().create_function(|_, ()| Ok(())).unwrap())
        .unwrap();
    rt.lua().globals().set("bad", bad).unwrap();
    rt.lua().scope(|scope| {
        rt.lua().globals().set("convert", scope.create_function(|_, value| codec.encode(value).map(|_| ()))?)?;
        rt.exec("local t={};for i=1,600000 do t[i]=true end;t[1]=bad;assert(not pcall(convert,t));local ok,err=pcall(convert,{[500000]=true});assert(not ok and tostring(err):find('data conversion budget',1,true))").unwrap();
        rt.exec("convert({[500000]=true})").unwrap();
        Ok(())
    }).unwrap();
}

#[test]
fn importing_and_exporting_share_the_same_outer_budget() {
    let rt = Runtime::new().unwrap();
    let codec = Codec::new(rt.lua()).unwrap();
    rt.lua()
        .scope(|scope| {
            rt.lua().globals().set(
                "roundtrip",
                scope.create_function(|_, value| {
                    let encoded = codec.encode(value)?;
                    codec.decode(encoded.data())
                })?,
            )?;
            rt.exec(
                "assert(not pcall(roundtrip,{[500000]=true}));assert(not pcall(roundtrip,true))",
            )
            .unwrap();
            rt.exec("assert(roundtrip(true))").unwrap();
            Ok(())
        })
        .unwrap();
}
