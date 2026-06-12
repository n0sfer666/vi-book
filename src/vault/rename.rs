use crate::markdown::extract::extract;
use crate::vault::link;

pub fn update_links(source: &str, old: &str, new: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let mut replacements: Vec<(usize, usize)> = Vec::new();
    for raw in extract(source).links {
        let parsed = link::parse(&raw.name);
        if parsed.is_some_and(|l| l.target == old)
            && let Some(span) = target_span(&chars, raw.start, raw.end)
        {
            replacements.push(span);
        }
    }
    let mut out: String = chars.iter().collect();
    for (start, end) in replacements.into_iter().rev() {
        let byte_start = char_to_byte(&chars, start);
        let byte_end = char_to_byte(&chars, end);
        out.replace_range(byte_start..byte_end, new);
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Affected {
    pub path: String,
    pub updated: String,
}

pub fn plan(sources: &[(String, String)], old: &str, new: &str) -> Vec<Affected> {
    sources
        .iter()
        .filter_map(|(path, source)| {
            let updated = update_links(source, old, new);
            (updated != *source).then(|| Affected {
                path: path.clone(),
                updated,
            })
        })
        .collect()
}

fn target_span(chars: &[char], link_start: usize, link_end: usize) -> Option<(usize, usize)> {
    let inner_start = link_start + 2;
    let inner_end = link_end.checked_sub(2)?;
    let mut start = inner_start;
    while start < inner_end && chars[start].is_whitespace() {
        start += 1;
    }
    let mut end = start;
    while end < inner_end && chars[end] != '#' && chars[end] != '|' {
        end += 1;
    }
    while end > start && chars[end - 1].is_whitespace() {
        end -= 1;
    }
    (end > start).then_some((start, end))
}

fn char_to_byte(chars: &[char], idx: usize) -> usize {
    chars[..idx].iter().map(|c| c.len_utf8()).sum()
}
