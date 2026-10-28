use v_ide_markdown::style::CodeToken;
use v_ide_markdown::syntax::highlight;
use v_ide_syntax::{Languages, set_dir, set_languages};

#[path = "../crates/syntax/tests/common/mod.rs"]
mod toy;

#[test]
fn only_installed_grammars_highlight_and_set_dir_invalidates_cached_config() {
    let root = toy::scratch("md-syntax-dylib-json");
    toy::install_toy(&root, "json");
    set_languages(Languages::builtin());
    set_dir(root);
    let code = "let x = 1;";
    let hl = highlight("json", code);
    assert_eq!(
        hl[0],
        Some(CodeToken::Keyword),
        "toy keyword via json.dylib"
    );
    assert_eq!(hl[8], Some(CodeToken::Number));
    set_dir(toy::scratch("md-syntax-dylib-empty"));
    for (fence, code) in [
        ("json", code),
        ("rust", "fn main() {}"),
        ("py", "def f(): pass"),
    ] {
        let hl = highlight(fence, code);
        assert!(hl.iter().all(Option::is_none), "{fence} without a grammar");
    }
}
