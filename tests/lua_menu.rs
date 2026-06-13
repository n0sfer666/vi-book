use vi_book::lua::host::PluginHost;

fn host_with(script: &str) -> PluginHost {
    let host = PluginHost::new().unwrap();
    host.load(script).unwrap();
    host
}

#[test]
fn menu_items_are_collected() {
    let host = host_with(
        "vibook.menu_item('g', 'Scratch', ':new scratch')\n\
         vibook.menu_item('r', 'Reindex', ':reindex')",
    );
    let items = host.menu_items();
    assert_eq!(items.len(), 2);
    assert_eq!(
        items[0],
        ("g".into(), "Scratch".into(), ":new scratch".into())
    );
    assert_eq!(items[1].2, ":reindex");
}

#[test]
fn options_read_back_through_opt() {
    let host = host_with(
        "vibook.opt.leader = ','\n\
         vibook.opt.insert_escape = 'jk'\n\
         vibook.opt.menu_position = 'center'",
    );
    assert_eq!(host.opt_string("leader").as_deref(), Some(","));
    assert_eq!(host.opt_string("insert_escape").as_deref(), Some("jk"));
    assert_eq!(host.opt_string("menu_position").as_deref(), Some("center"));
}
