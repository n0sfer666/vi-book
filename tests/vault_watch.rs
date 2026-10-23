mod common;

use std::fs;

use common::scratch;
use common::vault::{is_changed, strings, ws_at};
use common::watch::{
    alive, announced, announcements, facts_match, message, names_match, quiet, settle, started,
    temp_vault, vault, with_facts, with_names,
};
use v_ide_editor::module::{Event, Request};

#[test]
fn external_note_with_names_only_joins_names_silently() {
    let root = temp_vault("names");
    let (ws, mut modules) = with_names(&root);
    fs::write(root.join("sub/new.md"), "# New\n").unwrap();
    let ticks = settle(&ws, &mut modules, names_match(&root));
    assert!(ticks.iter().all(Vec::is_empty), "{ticks:?}");
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&["a.md", "sub/b.md", "sub/new.md"])
    );
}

#[test]
fn external_edit_with_facts_announces() {
    let root = temp_vault("edit");
    let (ws, mut modules) = with_facts(&root);
    fs::write(root.join("sub/b.md"), "no links #other\n").unwrap();
    let ticks = settle(&ws, &mut modules, facts_match(&root));
    assert!(announced(&ticks), "{ticks:?}");
    assert_eq!(vault(&modules).index().unwrap().tags(), strings(&["other"]));
}

#[test]
fn external_remove_drops_note() {
    let root = temp_vault("remove");
    let (ws, mut modules) = with_facts(&root);
    fs::remove_file(root.join("sub/b.md")).unwrap();
    let ticks = settle(&ws, &mut modules, facts_match(&root));
    assert!(announced(&ticks), "{ticks:?}");
    assert_eq!(vault(&modules).items().unwrap(), strings(&["a.md"]));
}

#[test]
fn external_rename_moves_note() {
    let root = temp_vault("rename");
    let (ws, mut modules) = with_facts(&root);
    fs::rename(root.join("a.md"), root.join("z.md")).unwrap();
    let ticks = settle(&ws, &mut modules, facts_match(&root));
    assert!(announced(&ticks), "{ticks:?}");
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&["sub/b.md", "z.md"])
    );
}

#[test]
fn case_only_rename_leaves_no_ghost() {
    let root = temp_vault("case");
    let (ws, mut modules) = with_facts(&root);
    fs::rename(root.join("a.md"), root.join("A.md")).unwrap();
    let ticks = settle(&ws, &mut modules, facts_match(&root));
    assert!(announced(&ticks), "{ticks:?}");
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&["A.md", "sub/b.md"])
    );
}

#[test]
fn unreadable_external_edit_keeps_facts_and_reports() {
    let root = temp_vault("unreadable");
    let (ws, mut modules) = with_facts(&root);
    fs::write(root.join("sub/b.md"), [0xff, 0xfe, 0x00]).unwrap();
    let text = message(&ws, &mut modules, "vault: sub/b.md: ");
    assert!(text.len() > "vault: sub/b.md: ".len(), "{text}");
    assert_eq!(vault(&modules).index().unwrap().tags(), strings(&["idea"]));
    assert_eq!(
        vault(&modules).items().unwrap(),
        strings(&["a.md", "sub/b.md"])
    );
}

#[test]
fn own_save_announces_once() {
    let root = temp_vault("own");
    let (ws, mut modules) = with_facts(&root);
    let abs = root.join("sub/b.md");
    fs::write(&abs, "now [[z]]\n").unwrap();
    let requests = modules.event(&ws, &Event::NoteSave(abs));
    assert!(requests.iter().any(is_changed), "{requests:?}");
    assert_eq!(quiet(&ws, &mut modules), Vec::new());
    assert_eq!(announcements(&alive(&root, &ws, &mut modules)), 1);
}

#[test]
fn unchanged_rewrite_is_silent() {
    let root = temp_vault("same");
    let (ws, mut modules) = with_facts(&root);
    fs::write(root.join("sub/b.md"), "see [[a]] #idea\n").unwrap();
    assert_eq!(quiet(&ws, &mut modules), Vec::new());
    assert_eq!(announcements(&alive(&root, &ws, &mut modules)), 1);
}

#[test]
fn burst_announces_at_most_once_per_tick() {
    let root = temp_vault("burst");
    let (ws, mut modules) = with_facts(&root);
    for name in ["c", "d", "e"] {
        fs::write(root.join(format!("{name}.md")), "[[a]] #burst\n").unwrap();
    }
    let ticks = settle(&ws, &mut modules, facts_match(&root));
    assert!(announced(&ticks), "{ticks:?}");
    let most = ticks
        .iter()
        .map(|tick| tick.iter().filter(|r| is_changed(r)).count())
        .max();
    assert_eq!(most, Some(1), "{ticks:?}");
}

#[test]
fn watch_failure_is_reported_once() {
    let root = scratch("vault-watch-absent").join("absent");
    let ws = ws_at(&root, None);
    let mut modules = started(&ws);
    assert!(vault(&modules).items().is_err());
    let text = message(&ws, &mut modules, "vault: watch: ");
    assert!(text.len() > "vault: watch: ".len(), "{text}");
    let again: Vec<Request> = quiet(&ws, &mut modules)
        .into_iter()
        .filter(|r| matches!(r, Request::Message(text) if text.starts_with("vault: watch: ")))
        .collect();
    assert_eq!(again, Vec::new());
}

#[cfg(target_os = "linux")]
#[test]
fn non_utf8_note_survives_external_edit() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let root = temp_vault("bytes");
    let abs = root.join(OsStr::from_bytes(b"n\xffe.md"));
    fs::write(&abs, "# N\n").unwrap();
    let (ws, mut modules) = with_facts(&root);
    fs::write(&abs, "body #edited\n").unwrap();
    let ticks = settle(&ws, &mut modules, facts_match(&root));
    assert!(announced(&ticks), "{ticks:?}");
    assert_eq!(
        vault(&modules).index().unwrap().tags(),
        strings(&["edited", "idea"])
    );
}
