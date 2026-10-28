use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::Deserialize;

use crate::languages_check::validate;
use crate::{LanguageSpec, LanguagesError};

const BUILTIN: &str = include_str!("../languages.toml");

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Languages {
    #[serde(default)]
    pub language: Vec<LanguageSpec>,
}

impl Languages {
    pub fn parse(text: &str) -> Result<Self, LanguagesError> {
        parse_text(text).map_err(LanguagesError::Parse)
    }

    pub fn load(config_dir: &Path) -> Result<Self, LanguagesError> {
        let path = config_dir.join("languages.toml");
        let text = match fs::read_to_string(&path) {
            Ok(text) => text,
            Err(err) if err.kind() == ErrorKind::NotFound => return Ok(Self::builtin()),
            Err(err) => return Err(LanguagesError::File(path, err.to_string())),
        };
        let merged = parse_text(&text).and_then(|custom| {
            let merged = Self::builtin().merge(custom);
            validate(&merged).map(|()| merged)
        });
        merged.map_err(|reason| LanguagesError::File(path, reason))
    }

    pub fn builtin() -> Self {
        Self::parse(BUILTIN).unwrap_or_default()
    }

    pub fn get(&self, name: &str) -> Option<&LanguageSpec> {
        self.language.iter().find(|spec| spec.name == name)
    }

    pub fn resolve(&self, fence: &str) -> Option<&LanguageSpec> {
        self.get(fence).or_else(|| {
            self.language
                .iter()
                .find(|spec| spec.aliases.iter().any(|alias| alias == fence))
        })
    }

    fn merge(mut self, custom: Self) -> Self {
        for spec in custom.language {
            match self.language.iter_mut().find(|own| own.name == spec.name) {
                Some(own) => *own = spec,
                None => self.language.push(spec),
            }
        }
        self
    }
}

fn parse_text(text: &str) -> Result<Languages, String> {
    let languages: Languages = toml::from_str(text).map_err(|err| err.to_string())?;
    validate(&languages).map(|()| languages)
}
