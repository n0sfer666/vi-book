use serde::Deserialize;
use v_ide_keys::MapMode;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub nvim_version: String,
    pub cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Case {
    pub name: String,
    pub before: String,
    pub cursor: [usize; 2],
    pub keys: String,
    #[serde(default)]
    pub maps: Vec<Map>,
    pub chunks: Option<Vec<String>>,
    pub screen: Option<[u16; 2]>,
    #[serde(default)]
    pub messages: Vec<String>,
    pub divergence: Option<String>,
    pub expect: Snapshot,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Map {
    #[serde(with = "MapModeDef")]
    pub mode: MapMode,
    pub lhs: String,
    pub rhs: String,
    pub noremap: bool,
    #[serde(default)]
    pub nowait: bool,
}

#[derive(Deserialize)]
#[serde(remote = "MapMode")]
enum MapModeDef {
    #[serde(rename = "n")]
    Normal,
    #[serde(rename = "i")]
    Insert,
    #[serde(rename = "x")]
    Visual,
    #[serde(rename = "o")]
    OpPending,
    #[serde(rename = "c")]
    Cmdline,
}

#[derive(Deserialize, PartialEq, Debug)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub text: String,
    pub cursor: [usize; 2],
    pub mode: String,
    pub unnamed: String,
}

pub const DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/oracle");

pub fn load(topic: &str) -> Fixture {
    let path = format!("{DIR}/{topic}.json");
    let raw = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{path}: {e}"));
    serde_json::from_str(&raw).unwrap_or_else(|e| panic!("{path}: {e}"))
}
