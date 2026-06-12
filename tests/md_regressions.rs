use insta::assert_snapshot;
use vi_book::markdown::render::render;

#[test]
fn wide_chars_table_aligns_by_display_width() {
    assert_snapshot!(render("| 日本 | b |\n|---|---|\n| x | 2 |", 20).text(), @r"
    ┌──────┬───┐
    │ 日本 │ b │
    ├──────┼───┤
    │ x    │ 2 │
    └──────┴───┘
    ");
}

#[test]
fn wide_chars_code_frame_aligns_by_display_width() {
    assert_snapshot!(render("```\n日本語\n```", 20).text(), @r"
    ┌──────────────────┐
    │ 日本語           │
    └──────────────────┘
    ");
}

#[test]
fn wide_chars_h1_rule_uses_display_width() {
    assert_snapshot!(render("# 日本", 11).text(), @"── 日本 ───");
}

#[test]
fn wide_chars_wrap_by_display_width() {
    assert_snapshot!(render("日本 語学 test", 6).text(), @r"
    日本
    語学
    test
    ");
}

#[test]
fn position_map_roundtrips_through_multibyte() {
    let rendered = render("x 日本 y", 40);
    assert_eq!(rendered.to_source(0, 2), Some(2));
    assert_eq!(rendered.to_rendered(3), (0, 3));
    assert_eq!(rendered.to_source(0, 5), Some(5));
}
