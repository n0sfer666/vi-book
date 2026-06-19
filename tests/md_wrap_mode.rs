use vi_book::markdown::render::render_with;
use vi_book::markdown::wrap_mode::WrapMode;

fn lines(src: &str, width: usize, mode: WrapMode) -> usize {
    render_with(src, width, mode).lines.len()
}

#[test]
fn none_keeps_paragraph_on_one_line() {
    assert_eq!(lines("aaa bbb ccc", 5, WrapMode::None), 1);
    assert_eq!(
        render_with("aaa bbb ccc", 5, WrapMode::None).text(),
        "aaa bbb ccc"
    );
}

#[test]
fn text_wraps_paragraph() {
    assert!(lines("aaa bbb ccc", 5, WrapMode::Text) > 1);
}

#[test]
fn all_wraps_paragraph() {
    assert!(lines("aaa bbb ccc", 5, WrapMode::All) > 1);
}

#[test]
fn none_keeps_list_item_on_one_line() {
    assert_eq!(lines("- item one two three", 8, WrapMode::None), 1);
}

#[test]
fn text_wraps_list_item() {
    assert!(lines("- item one two three", 8, WrapMode::Text) > 1);
}

#[test]
fn text_does_not_wrap_code_but_all_does() {
    let src = "```\nlonglongline\n```";
    let text_lines = lines(src, 8, WrapMode::Text);
    let all_lines = lines(src, 8, WrapMode::All);
    assert!(
        all_lines > text_lines,
        "all={all_lines} should wrap code more than text={text_lines}"
    );
    assert!(
        render_with(src, 8, WrapMode::Text)
            .text()
            .contains("longlongline"),
        "text mode keeps the code line intact"
    );
}

#[test]
fn text_does_not_wrap_heading_but_all_does() {
    let src = "## aaa bbb ccc ddd";
    let text_lines = lines(src, 6, WrapMode::Text);
    let all_lines = lines(src, 6, WrapMode::All);
    assert!(
        all_lines > text_lines,
        "all={all_lines} should wrap heading more than text={text_lines}"
    );
}

#[test]
fn table_never_wraps_in_any_mode() {
    let src = "| aaaaaa | bbbbbb |\n|---|---|\n| 1 | 2 |";
    let none = render_with(src, 6, WrapMode::None).text();
    let all = render_with(src, 6, WrapMode::All).text();
    assert_eq!(none, all, "table render is identical regardless of mode");
    assert!(all.contains("aaaaaa") && all.contains("bbbbbb"));
}

#[test]
fn none_keeps_quote_on_one_line() {
    assert_eq!(lines("> aaa bbb ccc ddd", 6, WrapMode::None), 1);
    assert!(lines("> aaa bbb ccc ddd", 6, WrapMode::Text) > 1);
}

#[test]
fn default_wrap_mode_is_none() {
    assert_eq!(WrapMode::default(), WrapMode::None);
}
