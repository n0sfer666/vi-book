mod common;

use std::fs;

use common::scratch;
use v_ide_syntax::{GrammarSource, LanguageSpec, Languages};

const TOY: &str = r#"
[[language]]
name = "toy"
aliases = ["ty", "t-2"]
extensions = ["toy"]
inherits = ["base"]

[language.grammar]
git = "https://example.invalid/toy"
rev = "0123456789abcdef0123456789abcdef01234567"
subpath = "sub"
"#;

const MINIMAL: &str = r#"
[[language]]
name = "toy"

[language.grammar]
git = "https://example.invalid/toy"
rev = "abc"
"#;

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(ToString::to_string).collect()
}

#[test]
fn parse_reads_every_field() {
    let languages = Languages::parse(TOY).unwrap();
    assert_eq!(
        languages.language,
        vec![LanguageSpec {
            name: "toy".to_string(),
            aliases: strings(&["ty", "t-2"]),
            extensions: strings(&["toy"]),
            inherits: strings(&["base"]),
            grammar: GrammarSource {
                git: "https://example.invalid/toy".to_string(),
                rev: "0123456789abcdef0123456789abcdef01234567".to_string(),
                subpath: Some("sub".to_string()),
            },
        }]
    );
}

#[test]
fn optional_fields_default_to_empty() {
    let languages = Languages::parse(MINIMAL).unwrap();
    let toy = languages.get("toy").unwrap();
    assert!(toy.aliases.is_empty() && toy.extensions.is_empty() && toy.inherits.is_empty());
    assert_eq!(toy.grammar.subpath, None);
}

#[test]
fn parse_rejects_missing_grammar_and_bad_names() {
    assert!(Languages::parse("[[language]]\nname = \"toy\"\n").is_err());
    assert!(Languages::parse(&MINIMAL.replace("\"toy\"", "\"../toy\"")).is_err());
    assert!(Languages::parse(&MINIMAL.replace("\"toy\"", "\"Toy\"")).is_err());
    assert!(Languages::parse("[[language]]\nname = 1\n").is_err());
}

#[test]
fn resolve_matches_name_or_alias_case_sensitively() {
    let languages = Languages::parse(TOY).unwrap();
    let name = |fence: &str| languages.resolve(fence).map(|spec| spec.name.clone());
    assert_eq!(name("toy").as_deref(), Some("toy"));
    assert_eq!(name("ty").as_deref(), Some("toy"));
    assert_eq!(name("t-2").as_deref(), Some("toy"));
    assert_eq!(name("Toy"), None);
    assert_eq!(name("TY"), None);
    assert_eq!(name("nope"), None);
    assert!(languages.get("ty").is_none());
}

#[test]
fn builtin_lists_twelve_languages_with_pinned_revs() {
    let languages = Languages::builtin();
    let names: Vec<&str> = languages
        .language
        .iter()
        .map(|spec| spec.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "rust",
            "python",
            "json",
            "bash",
            "javascript",
            "typescript",
            "tsx",
            "go",
            "c",
            "toml",
            "html",
            "yaml"
        ]
    );
    for spec in &languages.language {
        let rev = &spec.grammar.rev;
        assert!(
            rev.len() == 40 && rev.bytes().all(|b| b.is_ascii_hexdigit()),
            "{}: {rev}",
            spec.name
        );
        assert!(spec.grammar.git.starts_with("https://github.com/"));
    }
    let aliases = [
        ("rs", "rust"),
        ("py", "python"),
        ("sh", "bash"),
        ("shell", "bash"),
        ("zsh", "bash"),
        ("js", "javascript"),
        ("jsx", "javascript"),
        ("ts", "typescript"),
        ("golang", "go"),
        ("yml", "yaml"),
    ];
    for (alias, name) in aliases {
        assert_eq!(
            languages.resolve(alias).map(|spec| spec.name.as_str()),
            Some(name)
        );
    }
    for (name, subpath) in [("typescript", "typescript"), ("tsx", "tsx")] {
        let spec = languages.get(name).unwrap();
        assert_eq!(spec.inherits, strings(&["javascript"]));
        assert_eq!(spec.grammar.subpath.as_deref(), Some(subpath));
    }
}

#[test]
fn load_merges_override_by_name() {
    let config = scratch("languages-override");
    let replaced = MINIMAL.replacen("name = \"toy\"", "name = \"rust\"\naliases = [\"rr\"]", 1);
    let rust = Languages::parse(&replaced).unwrap().language;
    fs::write(
        config.join("languages.toml"),
        format!("{replaced}\n{MINIMAL}"),
    )
    .unwrap();
    let languages = Languages::load(&config).unwrap();
    let builtin = Languages::builtin();
    assert_eq!(languages.language.len(), builtin.language.len() + 1);
    assert_eq!(languages.language[0], rust[0]);
    assert_eq!(languages.language[1], builtin.language[1]);
    assert_eq!(
        languages.language.last().map(|spec| spec.name.as_str()),
        Some("toy")
    );
    assert!(languages.resolve("rs").is_none());
    assert_eq!(
        languages.resolve("rr").map(|spec| spec.name.as_str()),
        Some("rust")
    );
}

#[test]
fn load_without_override_is_builtin() {
    let config = scratch("languages-none");
    assert_eq!(Languages::load(&config).unwrap(), Languages::builtin());
}

#[test]
fn load_reports_broken_override_with_path() {
    let config = scratch("languages-broken");
    let path = config.join("languages.toml");
    fs::write(&path, "[[language]]\nname = 1\n").unwrap();
    let error = Languages::load(&config).unwrap_err().to_string();
    assert!(
        error.starts_with(&format!("{}: ", path.display())),
        "{error}"
    );
}

#[test]
fn parse_rejects_bad_aliases_unknown_keys_and_escaping_subpaths() {
    let alias = |alias: &str| {
        MINIMAL.replacen(
            "name = \"toy\"",
            &format!("name = \"toy\"\naliases = [\"{alias}\"]"),
            1,
        )
    };
    assert!(Languages::parse(&alias("ok-1")).is_ok());
    assert!(Languages::parse(&alias("Toy")).is_err());
    assert!(Languages::parse(&alias("t y")).is_err());
    assert!(Languages::parse(&alias("")).is_err());
    assert!(
        Languages::parse(&MINIMAL.replacen(
            "name = \"toy\"",
            "name = \"toy\"\nalias = [\"ty\"]",
            1
        ))
        .is_err()
    );
    assert!(Languages::parse(&format!("{MINIMAL}sub_path = \"x\"\n")).is_err());
    for subpath in ["../x", "a/../../x", "/abs"] {
        let text = format!("{MINIMAL}subpath = \"{subpath}\"\n");
        assert!(Languages::parse(&text).is_err(), "{subpath}");
    }
    assert!(Languages::parse(&format!("{MINIMAL}subpath = \"a/b\"\n")).is_ok());
}

#[test]
fn duplicate_names_or_aliases_are_rejected() {
    let other = MINIMAL.replace("\"toy\"", "\"other\"");
    let clash = |aliases: &str| {
        other.replacen(
            "name = \"other\"",
            &format!("name = \"other\"\naliases = {aliases}"),
            1,
        )
    };
    assert!(Languages::parse(&format!("{MINIMAL}\n{}", clash("[\"o\"]"))).is_ok());
    assert!(Languages::parse(&format!("{MINIMAL}\n{MINIMAL}")).is_err());
    assert!(Languages::parse(&format!("{MINIMAL}\n{}", clash("[\"toy\"]"))).is_err());
    assert!(Languages::parse(&format!("{TOY}\n{}", clash("[\"ty\"]"))).is_err());
    let config = scratch("languages-alias-clash");
    let path = config.join("languages.toml");
    for aliases in ["[\"sh\"]", "[\"rust\"]"] {
        fs::write(&path, clash(aliases)).unwrap();
        let error = Languages::load(&config).unwrap_err().to_string();
        assert!(
            error.starts_with(&format!("{}: ", path.display())),
            "{error}"
        );
    }
}
