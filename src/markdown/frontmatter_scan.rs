use crate::markdown::facts::Frontmatter;

pub fn scan(source: &str) -> Option<Frontmatter> {
    let mut lines = source.split_inclusive('\n');
    let first = lines.next()?;
    if first.trim_end_matches('\n') != "---" {
        return None;
    }
    let mut inner = String::new();
    let mut consumed = first.chars().count();
    for line in lines {
        let line_chars = line.chars().count();
        if line.trim_end_matches('\n') == "---" {
            return Some(Frontmatter {
                text: inner,
                end: consumed + line_chars,
            });
        }
        inner.push_str(line);
        consumed += line_chars;
    }
    None
}
