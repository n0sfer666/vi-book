use vi_book::markdown::render::render;
use vi_book::markdown::style::Style;

#[test]
fn list_items_wrap_to_the_column_width() {
    let rendered = render("- one two three four five six seven eight nine ten", 20);
    for line in &rendered.lines {
        assert!(
            line.text().chars().count() <= 20,
            "list line exceeds width: {:?}",
            line.text()
        );
    }
    assert!(rendered.lines.len() > 1, "a long item should wrap");
}

#[test]
fn list_items_render_inline_markup() {
    let rendered = render("- **bold** and `code`", 40);
    let text = rendered.text();
    assert!(
        !text.contains("**"),
        "bold markers should be concealed: {text:?}"
    );
    assert!(
        !text.contains('`'),
        "code backticks should be concealed: {text:?}"
    );
    assert!(text.contains("bold") && text.contains("code"));
    let styled_code = rendered
        .lines
        .iter()
        .flat_map(|line| &line.spans)
        .any(|span| span.style == Style::CodeSpan);
    assert!(styled_code, "inline code should be styled");
}

#[test]
fn second_list_item_starts_on_its_own_line() {
    let rendered = render("- first\n- second", 40);
    let lines: Vec<String> = rendered.lines.iter().map(|l| l.text()).collect();
    assert!(lines.iter().any(|l| l.contains("first")));
    assert!(lines.iter().any(|l| l.contains("second")));
    let first_line = lines.iter().position(|l| l.contains("first")).unwrap();
    let second_line = lines.iter().position(|l| l.contains("second")).unwrap();
    assert!(
        second_line > first_line,
        "items should be on separate lines"
    );
}
