use serde::{Deserialize, Serialize};

use crate::markdown::extract::extract;
use crate::vault::link::{self, Link};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    pub path: String,
    pub name: String,
    pub links: Vec<Link>,
    pub tags: Vec<String>,
    pub headings: Vec<String>,
}

impl Note {
    pub fn from_source(path: &str, source: &str) -> Self {
        let facts = extract(source);
        let links = facts
            .links
            .iter()
            .filter_map(|raw| link::parse(&raw.name))
            .collect();
        let tags = facts.tags.iter().map(|tag| tag.name.clone()).collect();
        Self {
            path: path.to_string(),
            name: stem(path),
            links,
            tags,
            headings: headings(source),
        }
    }
}

pub fn stem(path: &str) -> String {
    let file = path.rsplit('/').next().unwrap_or(path);
    file.strip_suffix(".md").unwrap_or(file).to_string()
}

fn headings(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let rest = line.trim_start_matches('#');
            let level = line.len() - rest.len();
            (level > 0 && rest.starts_with(' ')).then(|| rest.trim().to_string())
        })
        .collect()
}
