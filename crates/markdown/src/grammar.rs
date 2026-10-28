use tree_sitter::Language;

pub(crate) fn grammar(lang: &str) -> Option<(Language, String)> {
    v_ide_syntax::grammar(lang).map(|grammar| (grammar.language, grammar.highlights.to_string()))
}
