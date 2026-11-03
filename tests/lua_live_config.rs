mod common;
use v_ide_editor::document::Document;
use v_ide_tui::app::App;

#[test]
fn a_reported_config_failure_is_not_replaced_by_a_secondary_setting_error() {
    let mut app = App::new(Document::new("seed"));
    common::load_lua(
        &mut app,
        "reported-config-primary",
        r#"
   vibook.opt.leader=''
   vibook.create_autocmd('User',{callback=function()error('actual config failure')end})
   vibook.cmd('doautocmd <nomodeline> User')
 "#,
    );
    assert!(
        app.ui.message.contains("actual config failure"),
        "{}",
        app.ui.message
    );
    assert_eq!(
        app.ui
            .messages
            .history()
            .collect::<Vec<_>>()
            .join("\n")
            .matches("actual config failure")
            .count(),
        1
    );
}

#[test]
fn all_diagnostics_from_one_legacy_event_are_presented_together() {
    let mut app = App::new(Document::new("seed"));
    common::load_lua(
        &mut app,
        "reported-event-group",
        r#"
   vibook.create_autocmd('User',{callback=function()error('first event marker')end})
   vibook.create_autocmd('User',{callback=function()error('second event marker')end})
   vibook.on('follow_link',function()vibook.exec_autocmds('User',{modeline=false})end)
 "#,
    );
    common::draw(&mut app, 160, 60);
    v_ide_tui::requests::emit(
        &mut app,
        v_ide_editor::module::Event::FollowLink("bare".into()),
    );
    let output = app
        .ui
        .pager
        .as_ref()
        .unwrap()
        .visible()
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        output.contains("first event marker") && output.contains("second event marker"),
        "{output}"
    );
}
