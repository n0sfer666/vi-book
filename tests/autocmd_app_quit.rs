mod common;
use std::fs;
use v_ide_editor::document::Document;
use v_ide_tui::{app::App, command_line};

const EVENTS: &str = "QuitPre,ExitPre,VimLeavePre,VimLeave";

fn configure(app: &mut App, label: &str, extra: &str) {
    common::load_lua(
        app,
        label,
        &format!(
            r#"
   trace={{}}
   vibook.create_autocmd({{'QuitPre','ExitPre','VimLeavePre','VimLeave'}},{{callback=function(e)trace[#trace+1]=e.event end}})
   vibook.command('Capture',function()vibook.buf.set_text(table.concat(trace,','))end)
   {extra}
 "#
        ),
    );
}

fn trace(app: &mut App) -> String {
    command_line::run(app, "Capture");
    app.workspace.current().editor.text()
}

#[test]
fn quitting_delivers_shutdown_events_once_in_native_order() {
    let mut app = App::new(Document::new("seed"));
    configure(&mut app, "quit-event-order", "");
    command_line::run(&mut app, "q");
    assert!(app.quit);
    command_line::run(&mut app, "q!");
    assert_eq!(trace(&mut app), EVENTS);
}

#[test]
fn refused_quit_delivers_pre_events_and_force_delivers_the_leave_events() {
    let mut app = App::new(Document::new("seed"));
    configure(&mut app, "quit-dirty-events", "");
    app.workspace.current_mut().editor.replace_all("dirty");
    command_line::run(&mut app, "q");
    assert!(!app.quit);
    assert!(app.ui.message.contains("unsaved"));
    command_line::run(&mut app, "q!");
    assert!(app.quit);
    assert_eq!(trace(&mut app), format!("QuitPre,ExitPre,{EVENTS}"));
}

#[test]
fn quit_checks_unsaved_changes_after_pre_handlers() {
    let mut app = App::new(Document::new("seed"));
    configure(
        &mut app,
        "quit-handler-dirties",
        "vibook.create_autocmd('QuitPre',{callback=function()vibook.buf.set_text('changed')end})",
    );
    command_line::run(&mut app, "q");
    assert!(!app.quit);
    assert_eq!(app.workspace.current().editor.text(), "changed");
    assert_eq!(trace(&mut app), "QuitPre,ExitPre");
}

#[test]
fn wq_writes_before_quit_events_and_keeps_its_success_after_a_save_handler_edits() {
    let root = common::scratch("wq-native-events");
    let path = root.join("a.md");
    fs::write(&path, "disk").unwrap();
    let mut app = App::new(Document::open(&path).unwrap());
    configure(
        &mut app,
        "wq-native-events",
        &format!(
            r#"
   local expected={:?}
   vibook.create_autocmd('QuitPre',{{callback=function(e)assert(e.file==expected and e.match==expected);assert(vibook.buf.get_text()=='post save')end}})
   vibook.on('note_save',function()trace[#trace+1]='saved';vibook.buf.set_text('post save')end)
 "#,
            path.to_string_lossy()
        ),
    );
    app.workspace.current_mut().editor.replace_all("written");
    command_line::run(&mut app, "wq");
    assert_eq!(fs::read_to_string(path).unwrap(), "written");
    assert!(app.quit);
    assert!(app.workspace.current().is_dirty());
    assert_eq!(trace(&mut app), format!("saved,{EVENTS}"));
}

#[test]
fn shutdown_handler_errors_skip_later_phases_without_cancelling_a_clean_exit() {
    let stages: Vec<_> = EVENTS.split(',').collect();
    for (index, stage) in stages.iter().enumerate() {
        let mut app = App::new(Document::new("seed"));
        configure(
            &mut app,
            &format!("quit-error-{index}"),
            &format!(
                "vibook.create_autocmd('{stage}',{{callback=function()error('shutdown failure marker')end}})"
            ),
        );
        command_line::run(&mut app, "q");
        assert!(app.quit, "{stage}");
        let history = app.ui.messages.history().collect::<Vec<_>>().join("\n");
        assert_eq!(
            history.matches("shutdown failure marker").count(),
            1,
            "{history}"
        );
        assert_eq!(trace(&mut app), stages[..=index].join(","), "{stage}");
    }
}
