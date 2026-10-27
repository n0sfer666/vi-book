use tree_sitter::Language;

pub(crate) fn grammar(lang: &str) -> Option<(Language, String)> {
    v_ide_syntax::grammar(lang)
        .map(|grammar| (grammar.language, grammar.highlights.to_string()))
        .or_else(|| bundled(lang))
}

fn bundled(lang: &str) -> Option<(Language, String)> {
    let js = tree_sitter_javascript::HIGHLIGHT_QUERY;
    let (language_fn, query) = match lang {
        "rust" | "rs" => (
            tree_sitter_rust::LANGUAGE,
            tree_sitter_rust::HIGHLIGHTS_QUERY.to_string(),
        ),
        "python" | "py" => (
            tree_sitter_python::LANGUAGE,
            tree_sitter_python::HIGHLIGHTS_QUERY.to_string(),
        ),
        "json" => (
            tree_sitter_json::LANGUAGE,
            tree_sitter_json::HIGHLIGHTS_QUERY.to_string(),
        ),
        "bash" | "sh" | "shell" | "zsh" => (
            tree_sitter_bash::LANGUAGE,
            tree_sitter_bash::HIGHLIGHT_QUERY.to_string(),
        ),
        "javascript" | "js" | "jsx" => (tree_sitter_javascript::LANGUAGE, js.to_string()),
        "typescript" | "ts" => (
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
            format!("{js}\n{}", tree_sitter_typescript::HIGHLIGHTS_QUERY),
        ),
        "tsx" => (
            tree_sitter_typescript::LANGUAGE_TSX,
            format!("{js}\n{}", tree_sitter_typescript::HIGHLIGHTS_QUERY),
        ),
        "go" | "golang" => (
            tree_sitter_go::LANGUAGE,
            tree_sitter_go::HIGHLIGHTS_QUERY.to_string(),
        ),
        "c" => (
            tree_sitter_c::LANGUAGE,
            tree_sitter_c::HIGHLIGHT_QUERY.to_string(),
        ),
        "toml" => (
            tree_sitter_toml_ng::LANGUAGE,
            tree_sitter_toml_ng::HIGHLIGHTS_QUERY.to_string(),
        ),
        "html" => (
            tree_sitter_html::LANGUAGE,
            tree_sitter_html::HIGHLIGHTS_QUERY.to_string(),
        ),
        "yaml" | "yml" => (
            tree_sitter_yaml::LANGUAGE,
            tree_sitter_yaml::HIGHLIGHTS_QUERY.to_string(),
        ),
        _ => return None,
    };
    Some((Language::new(language_fn), query))
}
