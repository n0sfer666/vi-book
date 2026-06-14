use vi_book::markdown::render::render;
use vi_book::markdown::style::{CodeToken, Style};
use vi_book::markdown::syntax::highlight;

fn token_at(code: &str, needle: &str, hl: &[Option<CodeToken>]) -> Option<CodeToken> {
    let byte = code.find(needle).expect("needle present");
    let char_idx = code[..byte].chars().count();
    hl[char_idx]
}

#[test]
fn python_tokens_are_classified() {
    let code = "def f():\n    x = \"hi\"  # note\n    return 1\n";
    let hl = highlight("python", code);
    assert_eq!(hl.len(), code.chars().count());
    assert_eq!(hl[0], Some(CodeToken::Keyword), "def is a keyword");
    assert_eq!(token_at(code, "return", &hl), Some(CodeToken::Keyword));
    assert_eq!(token_at(code, "hi", &hl), Some(CodeToken::String));
    assert_eq!(token_at(code, "# note", &hl), Some(CodeToken::Comment));
    assert_eq!(token_at(code, "1\n", &hl), Some(CodeToken::Number));
}

#[test]
fn rust_keyword_is_classified() {
    let code = "fn main() {}\n";
    let hl = highlight("rust", code);
    assert_eq!(hl[0], Some(CodeToken::Keyword));
}

#[test]
fn tsx_is_highlighted() {
    let code = "const x = 1; // note\n";
    let hl = highlight("tsx", code);
    assert_eq!(hl.len(), code.chars().count());
    assert_eq!(token_at(code, "const", &hl), Some(CodeToken::Keyword));
    assert_eq!(token_at(code, "1;", &hl), Some(CodeToken::Number));
    assert_eq!(token_at(code, "// note", &hl), Some(CodeToken::Comment));
    assert!(
        hl.iter().filter(|t| t.is_some()).count() > 3,
        "tsx should produce varied tokens, not a single colour"
    );
}

#[test]
fn unknown_language_yields_no_tokens_but_correct_length() {
    let code = "some plain text";
    let hl = highlight("madeuplang", code);
    assert_eq!(hl.len(), code.chars().count());
    assert!(hl.iter().all(Option::is_none));
}

#[test]
fn rendered_python_code_block_carries_token_styles() {
    let rendered = render("```python\ndef f():\n    pass\n```", 40);
    let has_keyword = rendered
        .lines
        .iter()
        .flat_map(|line| &line.spans)
        .any(|span| span.style == Style::Code(CodeToken::Keyword));
    assert!(has_keyword, "code block should carry keyword token styles");
}
