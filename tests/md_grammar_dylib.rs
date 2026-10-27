use v_ide_markdown::style::CodeToken;
use v_ide_markdown::syntax::highlight;

#[path = "../crates/syntax/tests/common/mod.rs"]
mod toy;

#[test]
fn dylib_grammar_wins_and_set_dir_invalidates_cached_config() {
    let root = toy::scratch("md-syntax-dylib-json");
    toy::install_toy(&root, "json");
    v_ide_syntax::set_dir(root);
    let code = "let x = 1;";
    let hl = highlight("json", code);
    assert_eq!(
        hl[0],
        Some(CodeToken::Keyword),
        "toy keyword via json.dylib"
    );
    assert_eq!(hl[8], Some(CodeToken::Number));
    v_ide_syntax::set_dir(toy::scratch("md-syntax-dylib-empty"));
    let hl = highlight("json", code);
    assert_ne!(hl[0], Some(CodeToken::Keyword), "bundled json after reset");
}
