use crate::markdown::facts::{NoteFacts, Tag, WikiLink};
use crate::markdown::{frontmatter_scan, parser};

pub fn extract(source: &str) -> NoteFacts {
    let frontmatter = frontmatter_scan::scan(source);
    let body_start = frontmatter.as_ref().map_or(0, |fm| fm.end);
    let zones = parser::code_zones(source);
    let chars: Vec<char> = source.chars().collect();
    let mut facts = NoteFacts {
        frontmatter,
        ..NoteFacts::default()
    };
    let mut i = body_start;
    while i < chars.len() {
        if parser::in_zones(&zones, i) {
            i += 1;
            continue;
        }
        if let Some(link) = wiki_link_at(&chars, i) {
            i = link.end;
            facts.links.push(link);
            continue;
        }
        if let Some(tag) = tag_at(&chars, i, body_start) {
            i = tag.end;
            facts.tags.push(tag);
            continue;
        }
        i += 1;
    }
    facts
}

fn wiki_link_at(chars: &[char], start: usize) -> Option<WikiLink> {
    if chars.get(start) != Some(&'[') || chars.get(start + 1) != Some(&'[') {
        return None;
    }
    let mut j = start + 2;
    while j < chars.len() && chars[j] != '\n' {
        if chars[j] == ']' && chars.get(j + 1) == Some(&']') {
            let name: String = chars[start + 2..j].iter().collect();
            let name = name.trim().to_string();
            if name.is_empty() {
                return None;
            }
            return Some(WikiLink {
                name,
                start,
                end: j + 2,
            });
        }
        j += 1;
    }
    None
}

fn tag_at(chars: &[char], start: usize, body_start: usize) -> Option<Tag> {
    if chars.get(start) != Some(&'#') {
        return None;
    }
    if start > body_start && !chars[start - 1].is_whitespace() {
        return None;
    }
    let mut j = start + 1;
    while j < chars.len() && is_tag_char(chars[j]) {
        j += 1;
    }
    if j == start + 1 {
        return None;
    }
    let name: String = chars[start + 1..j].iter().collect();
    if name.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some(Tag {
        name,
        start,
        end: j,
    })
}

fn is_tag_char(c: char) -> bool {
    c.is_alphanumeric() || c == '-' || c == '_'
}
