use v_ide_markdown::render::render;
use v_ide_markdown::style::{CodeToken, Style};
use v_ide_markdown::syntax::highlight;

#[path = "../crates/syntax/tests/common/mod.rs"]
mod toy;

use toy::highlighting::toy_highlighting;

fn token_at(code: &str, needle: &str, hl: &[Option<CodeToken>]) -> Option<CodeToken> {
    let byte = code.find(needle).expect("needle present");
    let char_idx = code[..byte].chars().count();
    hl[char_idx]
}

#[test]
fn toy_tokens_are_classified() {
    toy_highlighting();
    let code = "let s = \"hé\"; # note\nlet n = 1;\n";
    let hl = highlight("toy", code);
    assert_eq!(hl.len(), code.chars().count());
    assert_eq!(hl[0], Some(CodeToken::Keyword));
    assert_eq!(token_at(code, "s =", &hl), Some(CodeToken::Variable));
    assert_eq!(token_at(code, "hé", &hl), Some(CodeToken::String));
    assert_eq!(token_at(code, "# note", &hl), Some(CodeToken::Comment));
    assert_eq!(token_at(code, "1;", &hl), Some(CodeToken::Number));
    assert_eq!(token_at(code, " = ", &hl), None);
}

#[test]
fn fence_resolves_through_alias_and_case() {
    toy_highlighting();
    let code = "let x = 1;";
    for fence in ["ty", "TOY", "Ty"] {
        assert_eq!(
            highlight(fence, code)[0],
            Some(CodeToken::Keyword),
            "{fence}"
        );
    }
}

#[test]
fn unknown_or_unbuilt_language_yields_no_tokens_but_correct_length() {
    toy_highlighting();
    let code = "let x = 1;";
    for fence in ["madeuplang", "ghost", "rust", ""] {
        let hl = highlight(fence, code);
        assert_eq!(hl.len(), code.chars().count(), "{fence}");
        assert!(hl.iter().all(Option::is_none), "{fence}");
    }
}

#[test]
fn rendered_toy_code_block_carries_token_styles() {
    toy_highlighting();
    let rendered = render("```toy\nlet s = \"x\";\n```", 40);
    let styles: Vec<Style> = rendered
        .lines
        .iter()
        .flat_map(|line| &line.spans)
        .map(|span| span.style)
        .collect();
    assert!(styles.contains(&Style::Code(CodeToken::Keyword)));
    assert!(styles.contains(&Style::Code(CodeToken::String)));
}
