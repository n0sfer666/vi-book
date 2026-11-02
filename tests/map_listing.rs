mod common;

use common::feed::app;
use v_ide_tui::command_line::run;

#[test]
fn common_definition_lists_once_and_filtered_view_retains_its_mode_group() {
    let mut app = app("text");
    run(&mut app, "map Q j");
    run(&mut app, "nmap Q");
    assert!(app.ui.message.starts_with("   Q"), "{}", app.ui.message);
    assert!(app.ui.message.ends_with("  j"), "{}", app.ui.message);
    assert_eq!(app.ui.message.lines().count(), 1);
    assert!(app.ui.pager.is_none());
}

#[test]
fn separate_definitions_and_partial_replacement_remain_distinct() {
    let mut app = app("text");
    run(&mut app, "map Q j");
    run(&mut app, "nnoremap Q k");
    run(&mut app, "map Q");
    let lines = app.ui.message.lines().collect::<Vec<_>>();
    assert_eq!(lines.len(), 2);
    assert!(
        lines[0].starts_with("n  Q") && lines[0].ends_with("* k"),
        "{}",
        app.ui.message
    );
    assert!(
        lines[1].starts_with("ox Q") && lines[1].ends_with("  j"),
        "{}",
        app.ui.message
    );
    assert!(app.ui.pager.is_some());
}

#[test]
fn listing_filters_prefix_and_formats_keys_commands_and_nop() {
    let mut app = app("text");
    run(&mut app, "nnoremap Qa <Nop>");
    run(&mut app, "nmap Qb j<Cmd>only<CR>k");
    run(&mut app, "nmap X dd");
    run(&mut app, "nmap Q");
    assert_eq!(app.ui.message.lines().count(), 2);
    assert!(app.ui.message.contains("* <Nop>"));
    assert!(app.ui.message.contains("j<Cmd>only<CR>k"));
    assert!(!app.ui.message.contains("dd"));
}

#[test]
fn bang_lists_grouped_insert_cmdline_maps_and_regular_insert_escape() {
    let mut app = app("text");
    run(&mut app, "noremap! qq <Esc>");
    run(&mut app, "cmap qq");
    assert!(app.ui.message.starts_with("!  qq"), "{}", app.ui.message);
    run(&mut app, "imap jj");
    assert!(
        app.ui.message.starts_with("i  jj") && app.ui.message.ends_with("* <Esc>"),
        "{}",
        app.ui.message
    );
    run(&mut app, "iunmap jj");
    run(&mut app, "imap jj");
    assert_eq!(app.ui.message, "No mapping found");
}
