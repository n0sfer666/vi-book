use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Link {
    pub target: String,
    pub alias: Option<String>,
    pub anchor: Option<String>,
}

impl Link {
    pub fn display(&self) -> &str {
        self.alias.as_deref().unwrap_or(&self.target)
    }
}

pub fn parse(raw: &str) -> Option<Link> {
    let (before, alias) = match raw.split_once('|') {
        Some((b, a)) => (b, non_empty(a)),
        None => (raw, None),
    };
    let (target, anchor) = match before.split_once('#') {
        Some((t, a)) => (t, non_empty(a)),
        None => (before, None),
    };
    let target = target.trim().to_string();
    if target.is_empty() {
        return None;
    }
    Some(Link {
        target,
        alias,
        anchor,
    })
}

fn non_empty(s: &str) -> Option<String> {
    let trimmed = s.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}
