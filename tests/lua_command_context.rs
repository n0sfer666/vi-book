mod common;
use std::fs;
use v_ide_editor::document::Document;
use v_ide_tui::{app::App, command_line};

#[test]
fn failed_nested_commands_discard_only_their_own_outbound_intents() {
    let root = common::scratch("nested-command-intents");
    fs::write(root.join("a.md"), "A").unwrap();
    fs::write(root.join("b.md"), "B").unwrap();
    let mut app = App::new(Document::new("seed"));
    app.workspace.set_cwd(root);
    common::load_lua(
        &mut app,
        "nested-command-intents",
        r#"
   vibook.command('Bad',function()vibook.open('b.md');error('expected inner failure')end)
   vibook.command('Catch',function()vibook.open('a.md');assert(not pcall(vibook.cmd,'Bad'))end)
 "#,
    );
    command_line::run(&mut app, "Catch");
    assert_eq!(app.workspace.current().editor.text(), "A");
}

#[test]
fn failed_native_callbacks_do_not_leak_outbound_intents_into_the_next_call() {
    let root = common::scratch("native-failed-intents");
    fs::write(root.join("b.md"), "B").unwrap();
    let mut app = App::new(Document::new("seed"));
    app.workspace.set_cwd(root);
    common::load_lua(
        &mut app,
        "native-failed-intents",
        r#"
   vibook.create_autocmd('User',{pattern='Fail',callback=function()vibook.open('b.md');error('expected failure')end})
   vibook.command('Noop',function()end)
 "#,
    );
    command_line::run(&mut app, "doautocmd <nomodeline> User Fail");
    command_line::run(&mut app, "Noop");
    assert_eq!(app.workspace.current().editor.text(), "seed");
}

#[test]
fn recursive_plugin_commands_fail_safely_and_restore_the_context() {
    let mut app = App::new(Document::new("seed"));
    common::load_lua(
        &mut app,
        "recursive-context-command",
        r#"
   vibook.command('Again',function()vibook.cmd('Again')end)
   vibook.command('Recover',function()vibook.buf.set_text('recovered')end)
 "#,
    );
    common::draw(&mut app, 160, 40);
    command_line::run(&mut app, "Again");
    assert!(
        app.ui.message.contains("stack overflow") || app.ui.message.contains("stack space"),
        "{}",
        app.ui.message
    );
    command_line::run(&mut app, "Recover");
    assert_eq!(app.workspace.current().editor.text(), "recovered");
}

#[test]
fn configuration_intents_are_discarded_before_the_first_command() {
    let root = common::scratch("config-command-intents");
    fs::write(root.join("b.md"), "B").unwrap();
    let mut app = App::new(Document::new("seed"));
    app.workspace.set_cwd(root);
    common::load_lua(
        &mut app,
        "config-command-intents",
        r#"
   vibook.open('b.md');vibook.command('Noop',function()end)
 "#,
    );
    command_line::run(&mut app, "Noop");
    assert_eq!(app.workspace.current().editor.text(), "seed");
}

#[test]
fn inline_commands_report_saved_errors_again_in_a_later_context() {
    let mut app = App::new(Document::new("seed"));
    common::load_lua(
        &mut app,
        "inline-saved-error",
        r#"
   vibook.create_autocmd('User',{pattern='Seed',callback=function()error('inline saved marker')end})
   local ok;ok,saved_error=pcall(vibook.cmd,'doautocmd <nomodeline> User Seed');assert(not ok)
   vibook.command('Rethrow',function()error(saved_error)end)
   vibook.create_autocmd('User',{pattern='Later',command='Rethrow'})
 "#,
    );
    app.ui.message.clear();
    command_line::run(&mut app, "doautocmd <nomodeline> User Later");
    assert!(
        app.ui.message.contains("inline saved marker"),
        "{}",
        app.ui.message
    );
}
