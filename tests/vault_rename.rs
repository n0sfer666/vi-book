use vi_book::vault::rename::{Affected, plan, update_links};

#[test]
fn rewrites_plain_link() {
    assert_eq!(
        update_links("see [[old]] here", "old", "new"),
        "see [[new]] here"
    );
}

#[test]
fn preserves_alias() {
    assert_eq!(update_links("[[old|label]]", "old", "new"), "[[new|label]]");
}

#[test]
fn preserves_anchor() {
    assert_eq!(
        update_links("[[old#Section]]", "old", "new"),
        "[[new#Section]]"
    );
}

#[test]
fn preserves_anchor_and_alias() {
    assert_eq!(
        update_links("[[old#Sec|lbl]]", "old", "new"),
        "[[new#Sec|lbl]]"
    );
}

#[test]
fn rewrites_multiple_occurrences() {
    assert_eq!(
        update_links("[[old]] and [[old|x]]", "old", "new"),
        "[[new]] and [[new|x]]"
    );
}

#[test]
fn leaves_other_targets_untouched() {
    assert_eq!(
        update_links("[[old]] [[other]]", "old", "new"),
        "[[new]] [[other]]"
    );
}

#[test]
fn does_not_touch_links_in_code() {
    assert_eq!(
        update_links("`[[old]]`\n[[old]]", "old", "new"),
        "`[[old]]`\n[[new]]"
    );
}

#[test]
fn preserves_surrounding_whitespace_in_link() {
    assert_eq!(update_links("[[ old ]]", "old", "new"), "[[ new ]]");
}

#[test]
fn rename_to_multiword_name() {
    assert_eq!(update_links("[[old]]", "old", "my note"), "[[my note]]");
}

#[test]
fn handles_multibyte_content_around_link() {
    assert_eq!(
        update_links("日本 [[old]] 語", "old", "new"),
        "日本 [[new]] 語"
    );
}

#[test]
fn unrelated_text_is_unchanged() {
    assert_eq!(update_links("no links here", "old", "new"), "no links here");
}

#[test]
fn plan_returns_only_changed_files() {
    let sources = vec![
        ("a.md".to_string(), "[[old]]".to_string()),
        ("b.md".to_string(), "no link".to_string()),
        ("c.md".to_string(), "[[old|x]] and [[keep]]".to_string()),
    ];
    let affected = plan(&sources, "old", "new");
    assert_eq!(
        affected,
        vec![
            Affected {
                path: "a.md".into(),
                updated: "[[new]]".into()
            },
            Affected {
                path: "c.md".into(),
                updated: "[[new|x]] and [[keep]]".into()
            },
        ]
    );
}
