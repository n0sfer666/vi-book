mod common;

use std::fs;

use common::{compile, install_toy, library, query, scratch, write_query};
use tree_sitter::Parser;
use v_ide_syntax::{LoadError, load};

#[test]
fn toy_grammar_loads_and_parses() {
    let root = scratch("loader-toy");
    install_toy(&root, "toy");
    let grammar = load(&root, "toy").unwrap();
    assert_eq!(&*grammar.highlights, common::HIGHLIGHTS);
    let mut parser = Parser::new();
    parser.set_language(&grammar.language).unwrap();
    let tree = parser.parse("let x = 1; # c", None).unwrap();
    assert_eq!(tree.root_node().kind(), "program");
    assert!(!tree.root_node().has_error());
}

#[test]
fn dash_in_name_maps_to_underscore_in_symbol() {
    let root = scratch("loader-dash");
    install_toy(&root, "toy-two");
    assert!(load(&root, "toy-two").is_ok());
}

#[test]
fn missing_library_is_not_found() {
    let root = scratch("loader-missing");
    let err = load(&root, "nope").unwrap_err();
    assert!(matches!(err, LoadError::NotFound(path) if path == library(&root, "nope")));
}

#[test]
fn missing_query_is_not_found() {
    let root = scratch("loader-no-query");
    install_toy(&root, "toy");
    fs::remove_file(query(&root, "toy")).unwrap();
    let err = load(&root, "toy").unwrap_err();
    assert!(matches!(err, LoadError::NotFound(path) if path == query(&root, "toy")));
}

#[test]
fn names_outside_the_charset_are_rejected() {
    let root = scratch("loader-names").join("inner");
    fs::create_dir_all(&root).unwrap();
    install_toy(&root.join(".."), "toy");
    for name in ["../toy", "a/b", "Toy", "", "c++", "toy.dylib"] {
        let err = load(&root, name).unwrap_err();
        assert!(matches!(err, LoadError::BadName(_)), "{name:?}: {err}");
    }
}

#[test]
fn library_without_symbol_is_rejected() {
    let root = scratch("loader-symbol");
    compile(&root, "nosym", "int other(void) { return 0; }\n", &[]);
    write_query(&root, "nosym", "");
    let err = load(&root, "nosym").unwrap_err();
    assert!(matches!(err, LoadError::Symbol(symbol) if symbol == "tree_sitter_nosym"));
}

#[test]
fn incompatible_abi_is_rejected() {
    let root = scratch("loader-abi");
    let source =
        "static const unsigned v[64] = {99};\nconst void *tree_sitter_old(void) { return v; }\n";
    compile(&root, "old", source, &[]);
    write_query(&root, "old", "");
    let err = load(&root, "old").unwrap_err();
    assert!(matches!(err, LoadError::Abi(99)), "{err}");
}

#[test]
fn corrupt_library_is_an_open_error() {
    let root = scratch("loader-corrupt");
    let path = library(&root, "junk");
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"not a library").unwrap();
    write_query(&root, "junk", "");
    assert!(matches!(load(&root, "junk"), Err(LoadError::Open(_))));
}

#[test]
fn broken_query_is_rejected() {
    let root = scratch("loader-query");
    install_toy(&root, "toy");
    write_query(&root, "toy", "(no_such_node) @keyword");
    assert!(matches!(load(&root, "toy"), Err(LoadError::Query(_))));
}

#[test]
fn null_language_is_rejected() {
    let root = scratch("loader-null");
    compile(
        &root,
        "nul",
        "const void *tree_sitter_nul(void) { return 0; }\n",
        &[],
    );
    write_query(&root, "nul", "");
    let err = load(&root, "nul").unwrap_err();
    assert!(matches!(err, LoadError::Symbol(symbol) if symbol == "tree_sitter_nul"));
}

#[test]
fn unreadable_query_is_a_query_error() {
    let root = scratch("loader-query-dir");
    install_toy(&root, "toy");
    let path = query(&root, "toy");
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    assert!(matches!(load(&root, "toy"), Err(LoadError::Query(_))));
}
