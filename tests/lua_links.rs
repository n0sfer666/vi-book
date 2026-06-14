use vi_book::lua::defaults::LINKS;
use vi_book::lua::host::PluginHost;

fn host_with_links(extra: &str) -> PluginHost {
    let host = PluginHost::new().unwrap();
    host.load(LINKS).unwrap();
    if !extra.is_empty() {
        host.load(extra).unwrap();
    }
    host
}

fn follow(host: &PluginHost, target: &str) -> (Vec<String>, Vec<String>) {
    host.fire_with("follow_link", target).unwrap();
    (host.take_opens(), host.take_external_opens())
}

#[test]
fn code_files_open_externally() {
    let host = host_with_links("");
    let (internal, external) = follow(&host, "main.rs");
    assert!(internal.is_empty());
    assert_eq!(external, vec!["main.rs".to_string()]);
}

#[test]
fn markdown_files_open_internally() {
    let host = host_with_links("");
    let (internal, external) = follow(&host, "readme.md");
    assert_eq!(internal, vec!["readme.md".to_string()]);
    assert!(external.is_empty());
}

#[test]
fn bare_names_defer_to_the_host_resolver() {
    let host = host_with_links("");
    let (internal, external) = follow(&host, "some-note");
    assert!(
        internal.is_empty(),
        "no extension defers to vi-book resolver"
    );
    assert!(external.is_empty());
}

#[test]
fn the_plugin_can_be_disabled() {
    let host = host_with_links("vibook.opt.plugins = { links = false }");
    let (internal, external) = follow(&host, "main.rs");
    assert!(internal.is_empty());
    assert!(external.is_empty());
}
