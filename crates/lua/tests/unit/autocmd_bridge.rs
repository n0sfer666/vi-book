use std::{cell::RefCell, rc::Rc};

use v_ide_editor::{
    autocmd::{Bus, EventName, GroupFilter, Names, Occurrence, Source},
    document::Document,
    execution::Context,
    workspace::Workspace,
};

use super::Native;
use crate::{config::Bridge, runtime::Runtime};

#[test]
fn callback_data_is_copied_while_lua_functions_keep_identity() {
    let rt = Rc::new(Runtime::new().unwrap());
    rt.exec(crate::config::PRELUDE).unwrap();
    let bridge = Rc::new(RefCell::new(Bridge::default()));
    crate::options::install(&rt, &bridge).unwrap();
    let bus = Rc::new(Bus::new(false));
    let native = Native::new(rt.clone(), bridge, bus.clone()).unwrap();
    rt.exec(
        r#"
        fn = function() return 17 end
        payload = {a={value=3}, fn=fn}
        calls = 0
        for i=1,2 do
            vibook.create_autocmd('User', {callback=function(a)
                assert(a.data.a.value == 3 and a.data.fn == fn and a.data.fn() == 17)
                a.data.a.value = 99
                calls = calls + 1
            end})
        end
    "#,
    )
    .unwrap();
    let encoded = native
        .state
        .codec
        .encode(rt.lua().globals().get("payload").unwrap())
        .unwrap();
    let mut workspace = Workspace::new(Document::new(""));
    let cx = Context::new(&mut workspace);
    let occurrence = Occurrence::new(
        EventName::parse("User").unwrap(),
        cx.buffer(0),
        Names::same("sample"),
    )
    .with_data(encoded.data().clone());
    assert_eq!(
        bus.dispatch(&cx, &occurrence, GroupFilter::All, Source::Explicit),
        2
    );
    assert!(cx.take_messages().is_empty());
    assert!(rt.eval_bool("calls == 2 and payload.a.value == 3").unwrap());
}

#[test]
fn bus_owns_registered_callbacks_until_deleted_and_vm_closures_do_not_keep_state_alive() {
    let rt = Rc::new(Runtime::new().unwrap());
    rt.exec(crate::config::PRELUDE).unwrap();
    let bridge = Rc::new(RefCell::new(Bridge::default()));
    crate::options::install(&rt, &bridge).unwrap();
    let bus = Rc::new(Bus::new(false));
    let native = Native::new(rt.clone(), bridge, bus.clone()).unwrap();
    let weak = Rc::downgrade(&native.state);
    rt.exec("vibook.create_autocmd('User', {callback=function() end})")
        .unwrap();
    drop(native);
    assert!(weak.upgrade().is_some());
    bus.clear(&Default::default());
    assert!(weak.upgrade().is_none());
    assert!(
        rt.exec("vibook.create_autocmd('User', {command=''})")
            .is_err()
    );
}
