use v_ide_editor::document::Document;
use v_ide_editor::module::{Modules, Request};
use v_ide_editor::workspace::Workspace;
use v_ide_markdown::module::MarkdownModule;
use v_ide_markdown::wrap_mode::WrapMode;

fn setup() -> (Modules, Workspace) {
    (
        Modules::new(vec![MarkdownModule::registration()]),
        Workspace::new(Document::new("")),
    )
}

fn wrap(modules: &Modules) -> Option<WrapMode> {
    modules.get::<MarkdownModule>().map(MarkdownModule::wrap)
}

fn message(text: &str) -> Option<Vec<Request>> {
    Some(vec![Request::Message(String::from(text))])
}

#[test]
fn markdown_filetype_activates_with_wrap_all() {
    let (mut modules, ws) = setup();
    modules.filetype(&ws, "rust");
    assert_eq!(wrap(&modules), None);
    modules.filetype(&ws, "markdown");
    assert_eq!(wrap(&modules), Some(WrapMode::All));
}

#[test]
fn wrap_command_activates_and_sets_mode() {
    let (mut modules, ws) = setup();
    assert_eq!(modules.command(&ws, "wrap", "text"), message("wrap: text"));
    assert_eq!(wrap(&modules), Some(WrapMode::Text));
}

#[test]
fn wrap_command_rejects_unknown_mode() {
    let (mut modules, ws) = setup();
    assert_eq!(
        modules.command(&ws, "wrap", "bogus"),
        message("unknown wrap mode: bogus (none|text|all)")
    );
    assert_eq!(wrap(&modules), Some(WrapMode::All));
}
