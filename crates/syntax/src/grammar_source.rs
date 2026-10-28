use serde::Deserialize;

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct GrammarSource {
    pub git: String,
    pub rev: String,
    pub subpath: Option<String>,
}
