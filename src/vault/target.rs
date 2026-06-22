use crate::vault::follow::link_at;
use crate::vault::link::Link;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Wiki(Link),
    Path(String),
    Url(String),
}

pub fn target_at(source: &str, cursor: usize) -> Option<Target> {
    if let Some(link) = link_at(source, cursor) {
        return Some(Target::Wiki(link));
    }
    let chars: Vec<char> = source.chars().collect();
    if let Some(dest) = markdown_dest_at(&chars, cursor) {
        return Some(classify(dest));
    }
    if let Some(url) = url_at(&chars, cursor) {
        return Some(Target::Url(url));
    }
    backtick_file_at(&chars, cursor).map(Target::Path)
}

fn classify(dest: String) -> Target {
    if is_url(&dest) {
        Target::Url(dest)
    } else {
        let path = dest.split('#').next().unwrap_or(&dest).trim().to_string();
        Target::Path(path)
    }
}

fn is_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

fn markdown_dest_at(chars: &[char], cursor: usize) -> Option<String> {
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '['
            && chars.get(i + 1) != Some(&'[')
            && let Some((dest, end)) = parse_md_link(chars, i)
        {
            if cursor >= i && cursor < end {
                return Some(dest);
            }
            i = end;
            continue;
        }
        i += 1;
    }
    None
}

fn parse_md_link(chars: &[char], start: usize) -> Option<(String, usize)> {
    let mut j = start + 1;
    while j < chars.len() && chars[j] != ']' && chars[j] != '\n' {
        j += 1;
    }
    if chars.get(j) != Some(&']') || chars.get(j + 1) != Some(&'(') {
        return None;
    }
    let mut k = j + 2;
    while k < chars.len() && chars[k] != ')' && chars[k] != '\n' {
        k += 1;
    }
    if chars.get(k) != Some(&')') {
        return None;
    }
    let dest: String = chars[j + 2..k].iter().collect();
    let dest = dest.trim().to_string();
    if dest.is_empty() {
        return None;
    }
    Some((dest, k + 1))
}

fn url_at(chars: &[char], cursor: usize) -> Option<String> {
    let n = chars.len();
    let mut i = 0;
    while i < n {
        if starts_with(chars, i, "http://") || starts_with(chars, i, "https://") {
            let mut end = i;
            while end < n && !is_url_boundary(chars[end]) {
                end += 1;
            }
            while end > i && is_trailing_punct(chars[end - 1]) {
                end -= 1;
            }
            if cursor >= i && cursor < end {
                return Some(chars[i..end].iter().collect());
            }
            i = end.max(i + 1);
            continue;
        }
        i += 1;
    }
    None
}

fn backtick_file_at(chars: &[char], cursor: usize) -> Option<String> {
    let n = chars.len();
    let mut i = 0;
    while i < n {
        if chars[i] == '`' {
            let mut j = i + 1;
            while j < n && chars[j] != '`' && chars[j] != '\n' {
                j += 1;
            }
            if chars.get(j) == Some(&'`') {
                if cursor > i && cursor < j {
                    let content: String = chars[i + 1..j].iter().collect();
                    return looks_like_file(&content).then(|| content.trim().to_string());
                }
                i = j + 1;
                continue;
            }
        }
        i += 1;
    }
    None
}

fn looks_like_file(s: &str) -> bool {
    let s = s.trim();
    !s.is_empty() && !s.chars().any(char::is_whitespace) && has_extension(s)
}

fn has_extension(s: &str) -> bool {
    match s.rfind('.') {
        Some(dot) => {
            let ext = &s[dot + 1..];
            !ext.is_empty() && ext.chars().all(char::is_alphanumeric)
        }
        None => false,
    }
}

fn starts_with(chars: &[char], at: usize, pat: &str) -> bool {
    pat.chars()
        .enumerate()
        .all(|(k, c)| chars.get(at + k) == Some(&c))
}

fn is_url_boundary(c: char) -> bool {
    c.is_whitespace() || matches!(c, ')' | ']' | '>' | '`' | '"' | '\'' | '|' | '<')
}

fn is_trailing_punct(c: char) -> bool {
    matches!(c, '.' | ',' | ';' | ':' | '!' | '?')
}
