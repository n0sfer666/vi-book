mod common;

use std::fs;
use std::path::Path;

use common::repo::{toy_languages, toy_repo};
use common::{HIGHLIGHTS, library, query, scratch};
use v_ide_syntax::install::{self, InstallError};
use v_ide_syntax::{Languages, load};

type Step = fn(&Languages, &Path, &[String], &mut dyn std::io::Write) -> Result<bool, InstallError>;

fn run(step: Step, languages: &Languages, data: &Path, names: &[&str]) -> (bool, String) {
    let names: Vec<String> = names.iter().map(ToString::to_string).collect();
    let mut out = Vec::new();
    let ok = step(languages, data, &names, &mut out).unwrap();
    (ok, String::from_utf8(out).unwrap())
}

fn read(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}

#[test]
fn fetch_build_check_install_toy_at_pinned_rev() {
    let repo = scratch("install-repo");
    let (url, rev) = toy_repo(&repo);
    let data = scratch("install-data");
    let languages = Languages::parse(&toy_languages(&url, &rev)).unwrap();
    let both = "toy ok\nkid ok\n2 ok, 0 failed\n";
    assert_eq!(
        run(install::fetch, &languages, &data, &["toy", "kid"]),
        (true, both.to_string())
    );
    assert_eq!(
        read(&data.join("sources/toy/queries/highlights.scm")),
        HIGHLIGHTS
    );
    assert_eq!(
        run(install::build, &languages, &data, &["toy", "kid"]),
        (true, both.to_string())
    );
    assert!(library(&data, "toy").is_file());
    assert_eq!(read(&query(&data, "toy")), HIGHLIGHTS);
    assert_eq!(
        read(&query(&data, "kid")),
        format!("{HIGHLIGHTS}\n{HIGHLIGHTS}")
    );
    assert!(load(&data, "kid").is_ok());
    let checked = format!("{}\n{both}", data.display());
    assert_eq!(run(install::check, &languages, &data, &[]), (true, checked));
}

#[test]
fn refetch_overwrites_sources() {
    let repo = scratch("install-refetch-repo");
    let (url, rev) = toy_repo(&repo);
    let data = scratch("install-refetch-data");
    let languages = Languages::parse(&toy_languages(&url, &rev)).unwrap();
    assert!(run(install::fetch, &languages, &data, &["toy"]).0);
    fs::write(data.join("sources/toy/queries/highlights.scm"), "edited").unwrap();
    assert!(run(install::fetch, &languages, &data, &["toy"]).0);
    assert_eq!(
        read(&data.join("sources/toy/queries/highlights.scm")),
        HIGHLIGHTS
    );
}

#[test]
fn unknown_and_unfetched_languages_fail_independently() {
    let data = scratch("install-unfetched");
    let languages = Languages::parse(&toy_languages("file:///missing", "abc")).unwrap();
    let (ok, out) = run(install::build, &languages, &data, &["nope", "toy"]);
    assert!(!ok);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], "nope failed: unknown language");
    assert!(lines[1].starts_with("toy failed: "), "{out}");
    assert_eq!(lines[2], "0 ok, 2 failed");
}

#[test]
fn check_without_names_reports_every_language() {
    let data = scratch("install-check-all");
    let languages = Languages::parse(&toy_languages("file:///missing", "abc")).unwrap();
    let (ok, out) = run(install::check, &languages, &data, &[]);
    assert!(!ok);
    let lines: Vec<&str> = out.lines().collect();
    assert_eq!(lines[0], data.display().to_string());
    assert!(lines[1].starts_with("toy failed: not found: "), "{out}");
    assert!(lines[2].starts_with("kid failed: not found: "), "{out}");
    assert_eq!(lines[3], "0 ok, 2 failed");
}

#[test]
fn fetch_reports_git_failure() {
    let data = scratch("install-fetch-fail");
    let missing = scratch("install-fetch-missing").join("nothing");
    let url = format!("file://{}", missing.display());
    let languages = Languages::parse(&toy_languages(&url, "abc")).unwrap();
    let (ok, out) = run(install::fetch, &languages, &data, &["toy"]);
    assert!(!ok);
    let first = out.lines().next().unwrap();
    assert!(
        first.len() > "toy failed: ".len() && first.starts_with("toy failed: "),
        "{out}"
    );
    assert!(out.ends_with("0 ok, 1 failed\n"), "{out}");
}

#[test]
fn fetch_never_treats_toml_values_as_git_options() {
    let repo = scratch("install-options-repo");
    let (url, rev) = toy_repo(&repo);
    let data = scratch("install-options-data");
    let marker = data.join("pwned");
    let inject = format!("--upload-pack=touch {}; git-upload-pack", marker.display());
    for (git, pin) in [
        (url.as_str(), inject.as_str()),
        (inject.as_str(), rev.as_str()),
    ] {
        let languages = Languages::parse(&toy_languages(git, pin)).unwrap();
        let (ok, out) = run(install::fetch, &languages, &data, &["toy"]);
        assert!(!ok, "{out}");
        assert!(!marker.exists(), "{git} {pin}");
    }
}

#[test]
fn names_are_validated_even_without_parse() {
    let data = scratch("install-bad-name");
    let mut languages = Languages::parse(&toy_languages("file:///missing", "abc")).unwrap();
    languages.language[0].name = "../escape".to_string();
    let (ok, out) = run(install::check, &languages, &data, &["../escape"]);
    assert!(!ok);
    assert_eq!(
        out.lines().nth(1),
        Some("../escape failed: invalid language name")
    );
}

#[test]
fn parents_and_subpaths_are_validated_even_without_parse() {
    let repo = scratch("install-bad-paths-repo");
    let (url, rev) = toy_repo(&repo);
    let data = scratch("install-bad-paths-data");
    let base = Languages::parse(&toy_languages(&url, &rev)).unwrap();
    assert!(run(install::fetch, &base, &data, &["toy", "kid"]).0);
    let mut escaping = base.clone();
    escaping.language[0].grammar.subpath = Some("../../outside".to_string());
    for step in [install::fetch as Step, install::build, install::check] {
        let (ok, out) = run(step, &escaping, &data, &["toy"]);
        assert!(!ok);
        assert!(out.contains("toy failed: invalid subpath\n"), "{out}");
    }
    let mut parent = base;
    parent.language[0].name = "../toy".to_string();
    parent.language[1].inherits = vec!["../toy".to_string()];
    let (ok, out) = run(install::build, &parent, &data, &["kid"]);
    assert!(!ok);
    assert_eq!(
        out.lines().next(),
        Some("kid failed: invalid language name")
    );
}
