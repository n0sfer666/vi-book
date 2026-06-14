use vi_book::lua::config::ColorSpec;
use vi_book::lua::host::PluginHost;

fn host_with(script: &str) -> PluginHost {
    let host = PluginHost::new().unwrap();
    host.load(script).unwrap();
    host
}

#[test]
fn colors_are_collected_from_config() {
    let host = host_with(
        "vibook.color('keyword', { fg = 'red', bold = true })\n\
         vibook.color('comment', { fg = '#808080', italic = true })",
    );
    let colors = host.colors();
    assert_eq!(colors.len(), 2);
    assert_eq!(
        colors[0],
        ColorSpec {
            name: "keyword".into(),
            fg: Some("red".into()),
            bold: true,
            ..ColorSpec::default()
        }
    );
    assert_eq!(colors[1].name, "comment");
    assert_eq!(colors[1].fg.as_deref(), Some("#808080"));
    assert!(colors[1].italic);
    assert!(!colors[1].bold);
}
