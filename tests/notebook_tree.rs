use vi_book::notebook::tree::{Row, Tree};

fn row(path: &str, label: &str, depth: usize, is_dir: bool) -> Row {
    Row {
        path: path.into(),
        label: label.into(),
        depth,
        is_dir,
        collapsed: false,
    }
}

fn paths(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| s.to_string()).collect()
}

#[test]
fn flattens_directories_before_files_alphabetically() {
    let tree = Tree::from_paths(&paths(&["zeta.md", "docs/a.md", "docs/b.md", "alpha.md"]));
    assert_eq!(
        tree.flatten(),
        vec![
            row("docs", "docs", 0, true),
            row("docs/a.md", "a.md", 1, false),
            row("docs/b.md", "b.md", 1, false),
            row("alpha.md", "alpha.md", 0, false),
            row("zeta.md", "zeta.md", 0, false),
        ]
    );
}

#[test]
fn nested_directories_have_increasing_depth() {
    let tree = Tree::from_paths(&paths(&["a/b/c.md"]));
    assert_eq!(
        tree.flatten(),
        vec![
            row("a", "a", 0, true),
            row("a/b", "b", 1, true),
            row("a/b/c.md", "c.md", 2, false),
        ]
    );
}

#[test]
fn collapsing_a_directory_hides_its_subtree() {
    let mut tree = Tree::from_paths(&paths(&["docs/a.md", "docs/sub/b.md", "top.md"]));
    tree.toggle("docs");
    let flat = tree.flatten();
    assert_eq!(flat.len(), 2);
    assert_eq!(flat[0].path, "docs");
    assert!(flat[0].collapsed);
    assert_eq!(flat[1].path, "top.md");
}

#[test]
fn toggling_twice_restores_subtree() {
    let mut tree = Tree::from_paths(&paths(&["docs/a.md"]));
    tree.toggle("docs");
    tree.toggle("docs");
    assert!(!tree.is_collapsed("docs"));
    assert_eq!(tree.flatten().len(), 2);
}

#[test]
fn collapsing_inner_directory_only_hides_its_children() {
    let mut tree = Tree::from_paths(&paths(&["a/b/c.md", "a/d.md"]));
    tree.toggle("a/b");
    let flat = tree.flatten();
    assert_eq!(
        flat.iter().map(|r| r.path.as_str()).collect::<Vec<_>>(),
        vec!["a", "a/b", "a/d.md"]
    );
}

#[test]
fn empty_vault_has_empty_tree() {
    assert!(Tree::from_paths(&[]).flatten().is_empty());
}
