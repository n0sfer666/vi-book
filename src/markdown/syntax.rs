use tree_sitter::Language;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

use crate::markdown::style::CodeToken;

const NAMES: &[(&str, CodeToken)] = &[
    ("keyword", CodeToken::Keyword),
    ("string", CodeToken::String),
    ("number", CodeToken::Number),
    ("comment", CodeToken::Comment),
    ("function", CodeToken::Function),
    ("type", CodeToken::Type),
    ("constructor", CodeToken::Type),
    ("constant", CodeToken::Constant),
    ("constant.builtin", CodeToken::Constant),
    ("variable", CodeToken::Variable),
    ("operator", CodeToken::Operator),
    ("punctuation", CodeToken::Punctuation),
    ("property", CodeToken::Property),
    ("tag", CodeToken::Tag),
    ("attribute", CodeToken::Attribute),
];

fn grammar(lang: &str) -> Option<(Language, String)> {
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

fn configure(lang: &str) -> Option<HighlightConfiguration> {
    let (language, query) = grammar(lang)?;
    let query = query.as_str();
    let recognized: Vec<&str> = NAMES.iter().map(|(name, _)| *name).collect();
    let mut config = HighlightConfiguration::new(language, lang, query, "", "").ok()?;
    config.configure(&recognized);
    Some(config)
}

pub fn highlight(lang: &str, code: &str) -> Vec<Option<CodeToken>> {
    let char_count = code.chars().count();
    let blank = || vec![None; char_count];
    let Some(config) = configure(&lang.to_lowercase()) else {
        return blank();
    };
    let mut highlighter = Highlighter::new();
    let Ok(events) = highlighter.highlight(&config, code.as_bytes(), None, |_| None) else {
        return blank();
    };
    let mut by_byte = vec![None; code.len()];
    let mut stack: Vec<CodeToken> = Vec::new();
    for event in events {
        match event {
            Ok(HighlightEvent::HighlightStart(highlight)) => stack.push(NAMES[highlight.0].1),
            Ok(HighlightEvent::HighlightEnd) => {
                stack.pop();
            }
            Ok(HighlightEvent::Source { start, end }) => {
                if let Some(&token) = stack.last() {
                    for slot in by_byte.iter_mut().take(end).skip(start) {
                        *slot = Some(token);
                    }
                }
            }
            Err(_) => return blank(),
        }
    }
    code.char_indices().map(|(byte, _)| by_byte[byte]).collect()
}
