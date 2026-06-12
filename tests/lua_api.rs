use vi_book::lua::host::PluginHost;

#[test]
fn options_are_collected() {
    let host = PluginHost::new().unwrap();
    host.load("vibook.opt.wrap = true\nvibook.opt.theme = 'dark'")
        .unwrap();
    assert_eq!(host.opt_bool("wrap"), Some(true));
    assert_eq!(host.opt_string("theme"), Some("dark".to_string()));
    assert_eq!(host.opt_bool("missing"), None);
}

#[test]
fn commands_are_registered_and_runnable() {
    let host = PluginHost::new().unwrap();
    host.set_buffer("hello", 0);
    host.load(
        "vibook.command('shout', function()\n  vibook.buf.set_text(vibook.buf.get_text() .. '!')\nend)",
    )
    .unwrap();
    assert_eq!(host.command_names(), vec!["shout".to_string()]);
    assert!(host.run_command("shout").unwrap());
    assert_eq!(host.buffer_text(), "hello!");
}

#[test]
fn running_unknown_command_returns_false() {
    let host = PluginHost::new().unwrap();
    assert!(!host.run_command("nope").unwrap());
}

#[test]
fn event_handlers_fire() {
    let host = PluginHost::new().unwrap();
    host.set_buffer("x", 0);
    host.load("vibook.on('note_open', function()\n  vibook.buf.set_text('opened')\nend)")
        .unwrap();
    host.fire("note_open").unwrap();
    assert_eq!(host.buffer_text(), "opened");
}

#[test]
fn multiple_handlers_all_run() {
    let host = PluginHost::new().unwrap();
    host.set_buffer("", 0);
    host.load(
        "vibook.on('e', function() vibook.buf.set_text(vibook.buf.get_text() .. 'a') end)\n\
         vibook.on('e', function() vibook.buf.set_text(vibook.buf.get_text() .. 'b') end)",
    )
    .unwrap();
    host.fire("e").unwrap();
    assert_eq!(host.buffer_text(), "ab");
}

#[test]
fn keymaps_are_collected() {
    let host = PluginHost::new().unwrap();
    host.load("vibook.keymap('n', '<C-s>', ':w')").unwrap();
    let maps = host.keymaps();
    assert_eq!(maps.len(), 1);
    assert_eq!(maps[0].mode, "n");
    assert_eq!(maps[0].lhs, "<C-s>");
    assert_eq!(maps[0].rhs, ":w");
}

#[test]
fn vault_queries_see_injected_data() {
    let host = PluginHost::new().unwrap();
    host.set_vault(vec!["a.md".into(), "b.md".into()], vec!["tag1".into()]);
    host.set_backlinks("b.md", vec!["a.md".into()]);
    host.set_buffer("", 0);
    host.load(
        "vibook.command('report', function()\n\
           local n = vibook.vault.notes()\n\
           local b = vibook.vault.backlinks('b.md')\n\
           local t = vibook.vault.tags()\n\
           vibook.buf.set_text(#n .. ',' .. #b .. ',' .. t[1])\n\
         end)",
    )
    .unwrap();
    host.run_command("report").unwrap();
    assert_eq!(host.buffer_text(), "2,1,tag1");
}

#[test]
fn vault_create_and_open_record_intents() {
    let host = PluginHost::new().unwrap();
    host.load(
        "vibook.command('go', function()\n\
           vibook.vault.create('new note')\n\
           vibook.vault.open('a.md')\n\
         end)",
    )
    .unwrap();
    host.run_command("go").unwrap();
    assert_eq!(host.creates(), vec!["new note".to_string()]);
    assert_eq!(host.opens(), vec!["a.md".to_string()]);
}

#[test]
fn plugin_runtime_error_is_reported_not_panicked() {
    let host = PluginHost::new().unwrap();
    host.load("vibook.command('bad', function() error('boom') end)")
        .unwrap();
    assert!(host.run_command("bad").is_err());
}

#[test]
fn plugin_cannot_access_os_by_default() {
    let host = PluginHost::new().unwrap();
    host.set_buffer("", 0);
    host.load("vibook.command('c', function() vibook.buf.set_text(tostring(os)) end)")
        .unwrap();
    host.run_command("c").unwrap();
    assert_eq!(host.buffer_text(), "nil");
}
