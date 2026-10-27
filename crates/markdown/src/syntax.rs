use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard, OnceLock, PoisonError};

use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

#[cfg(feature = "highlight")]
use crate::grammar::grammar;
use crate::style::CodeToken;
#[cfg(feature = "highlight")]
use v_ide_syntax::generation;

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

#[cfg(not(feature = "highlight"))]
fn grammar(_lang: &str) -> Option<(tree_sitter::Language, String)> {
    None
}

#[cfg(not(feature = "highlight"))]
fn generation() -> u64 {
    0
}

fn configure(lang: &str) -> Option<HighlightConfiguration> {
    let (language, query) = grammar(lang)?;
    let query = query.as_str();
    let recognized: Vec<&str> = NAMES.iter().map(|(name, _)| *name).collect();
    let mut config = HighlightConfiguration::new(language, lang, query, "", "").ok()?;
    config.configure(&recognized);
    Some(config)
}

#[derive(Default)]
struct Configs {
    generation: u64,
    by_lang: HashMap<String, Option<Arc<HighlightConfiguration>>>,
}

static CONFIGS: OnceLock<Mutex<Configs>> = OnceLock::new();

fn configs() -> MutexGuard<'static, Configs> {
    CONFIGS
        .get_or_init(Mutex::default)
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

fn configured(lang: &str) -> Option<Arc<HighlightConfiguration>> {
    let generation = generation();
    {
        let mut configs = configs();
        if configs.generation != generation {
            configs.generation = generation;
            configs.by_lang.clear();
        }
        if let Some(cached) = configs.by_lang.get(lang) {
            return cached.clone();
        }
    }
    let config = configure(lang).map(Arc::new);
    let mut configs = configs();
    if configs.generation == generation {
        configs
            .by_lang
            .entry(lang.to_string())
            .or_insert_with(|| config.clone());
    }
    config
}

pub fn highlight(lang: &str, code: &str) -> Vec<Option<CodeToken>> {
    let char_count = code.chars().count();
    let blank = || vec![None; char_count];
    let Some(config) = configured(&lang.to_lowercase()) else {
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
