mod common;

use v_ide_lua::host::PluginHost;
use v_ide_tui::{app::App, command_line};

#[test]
fn scalar_and_object_interfaces_share_state_with_native_scope_rules() {
    let host = PluginHost::new().unwrap();
    host.load(
        r#"
        assert(vibook.o.timeoutlen == 1000)
        vibook.wo.number = true
        assert(vibook.go.number and vibook.o.number)
        vibook.bo.readonly = true
        assert(vibook.o.readonly and not vibook.go.readonly)
        vibook.opt_global.number = false
        assert(vibook.opt_local.number:get() and not vibook.opt_global.number:get())
        vibook.opt_local.number = false
        vibook.opt.readonly = false
        assert(not vibook.bo.readonly and not vibook.go.readonly)
        vibook.opt_local.timeoutlen = 73
        assert(vibook.go.timeoutlen == 73)
        vibook.opt.leader = "gg"
        assert(vibook.opt.leader == "gg")
    "#,
    )
    .unwrap();
}

#[test]
fn option_objects_capture_values_and_support_numeric_methods_and_operators() {
    let host = PluginHost::new().unwrap();
    host.load(
        r#"
        local nu = vibook.opt.number
        vibook.opt.number = true
        assert(not nu:get() and vibook.opt.number:get())
        local tm = vibook.opt.timeoutlen
        vibook.opt.timeoutlen = 40
        tm:append(7)
        assert(vibook.o.timeoutlen == 1007)
        vibook.opt.timeoutlen = vibook.opt.timeoutlen + 2
        vibook.opt.timeoutlen:remove(9)
        assert(vibook.o.timeoutlen == 1000)
        vibook.opt.timeoutlen = vibook.opt.timeoutlen - 1
        assert(vibook.opt.timeoutlen:get() == 999)
        assert(not pcall(function() vibook.opt.timeoutlen:prepend(2) end))
        assert(not pcall(function() vibook.opt_global.timeoutlen = vibook.opt.timeoutlen end))
    "#,
    )
    .unwrap();
}

#[test]
fn invalid_lua_values_or_wrong_scopes_raise_without_changing_options() {
    let host = PluginHost::new().unwrap();
    for code in [
        "vibook.o.timeoutlen = 1.5",
        "vibook.o.timeoutlen = '27'",
        "vibook.opt.timeoutlen = '27'",
        "vibook.o.number = 1",
        "vibook.opt.number = 1",
        "vibook.bo.number = true",
        "return vibook.wo.readonly",
        "vibook.wo.timeoutlen = 1",
        "vibook.o.unknown = true",
        "vibook.opt_local.unknown = true",
        "vibook.o.timeoutlen = -1",
        "vibook.opt.timeoutlen = 2147483648",
    ] {
        assert!(host.load(code).is_err(), "{code}");
    }
    host.load("assert(vibook.o.timeoutlen == 1000 and not vibook.o.number)")
        .unwrap();
}

#[test]
fn config_changes_reach_core_and_runtime_callbacks_see_current_ex_values() {
    let mut app = App::new(common::named_doc("one", "a.txt"));
    common::load_lua(
        &mut app,
        "core-option-sync",
        r#"
        vibook.opt.timeoutlen = 37
        vibook.opt_local.readonly = true
        vibook.command('check_options', function()
            assert(vibook.o.timeoutlen == 91)
            assert(vibook.bo.readonly)
            vibook.wo.number = false
            vibook.bo.readonly = false
            vibook.o.timeoutlen = 0
        end)
    "#,
    );
    command_line::run(&mut app, "set tm? ro?");
    assert_eq!(app.ui.message, "  timeoutlen=37\n  readonly");
    command_line::run(&mut app, "set tm=91");
    command_line::run(&mut app, "check_options");
    command_line::run(&mut app, "set nu? ro? tm?");
    assert_eq!(app.ui.message, "nonumber\nnoreadonly\n  timeoutlen=0");
    common::feed::map(&mut app, v_ide_keys::MapMode::Normal, "Q", "j", false);
    common::feed::map(&mut app, v_ide_keys::MapMode::Normal, "QQ", "k", false);
    common::feed::press(&mut app, "Q");
    assert!(app.typeahead.deadline().is_none());
}

#[test]
fn valid_option_changes_before_a_lua_error_are_kept() {
    let mut app = App::new(common::named_doc("one", "a.txt"));
    common::load_lua(
        &mut app,
        "core-option-error-prefix",
        r#"
        vibook.o.timeoutlen = 29
        vibook.opt.number = false
        error('stop config')
    "#,
    );
    assert!(app.ui.message.contains("stop config"));
    command_line::run(&mut app, "set tm? nu?");
    assert_eq!(app.ui.message, "  timeoutlen=29\nnonumber");
    common::load_lua(
        &mut app,
        "core-option-callback-error",
        r#"
        vibook.command('fail_options', function()
            vibook.o.timeoutlen = 43
            error('stop callback')
        end)
    "#,
    );
    command_line::run(&mut app, "fail_options");
    assert!(app.ui.message.contains("stop callback"));
    command_line::run(&mut app, "set tm?");
    assert_eq!(app.ui.message, "  timeoutlen=43");
}

#[test]
fn wrapper_arithmetic_defers_range_checks_and_assignment_requires_the_same_accessor_and_name() {
    let host = PluginHost::new().unwrap();
    host.load(
        r#"
        local value = vibook.opt.timeoutlen - 2000
        assert(value:get() == -1000 and vibook.o.timeoutlen == 1000)
        assert(not pcall(function() vibook.opt.timeoutlen = value end))
        assert(vibook.o.timeoutlen == 1000)
        local tm = vibook.opt.tm
        assert(not pcall(function() tm:remove(2000) end))
        assert(tm:get() == -1000 and vibook.o.timeoutlen == 1000)
        assert((vibook.opt.tm + 0.5):get() == 1000.5)
        assert(not pcall(function() return vibook.opt.tm + vibook.opt.tm end))
        assert(not pcall(function() return vibook.opt.tm ^ 2 end))
        assert(not pcall(function() vibook.opt.timeoutlen = vibook.opt.tm end))
        assert(not pcall(function() vibook.opt.number = vibook.opt.readonly end))
    "#,
    )
    .unwrap();
}
