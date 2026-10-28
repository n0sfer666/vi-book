use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use crate::{Grammar, Languages, LoadError, default_config_dir, default_dir, load};

struct Registry {
    dir: Option<PathBuf>,
    languages: Languages,
    generation: u64,
    grammars: HashMap<String, Option<Grammar>>,
}

impl Registry {
    fn reset(&mut self) {
        self.generation += 1;
        self.grammars.clear();
    }
}

static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();

fn registry() -> MutexGuard<'static, Registry> {
    REGISTRY
        .get_or_init(|| {
            Mutex::new(Registry {
                dir: default_dir(),
                languages: default_languages(),
                generation: 0,
                grammars: HashMap::new(),
            })
        })
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

fn default_languages() -> Languages {
    default_config_dir()
        .and_then(|dir| Languages::load(&dir).ok())
        .unwrap_or_else(Languages::builtin)
}

pub fn canonical(fence: &str) -> Option<String> {
    registry()
        .languages
        .resolve(fence)
        .map(|spec| spec.name.clone())
}

pub fn grammar(fence: &str) -> Option<Grammar> {
    let (lang, dir, generation) = {
        let registry = registry();
        let lang = registry.languages.resolve(fence)?.name.clone();
        if let Some(cached) = registry.grammars.get(&lang) {
            return cached.clone();
        }
        (lang, registry.dir.clone(), registry.generation)
    };
    let loaded = match dir.as_deref().map(|dir| load(dir, &lang)) {
        Some(Err(LoadError::BadName(_))) => return None,
        loaded => loaded.and_then(Result::ok),
    };
    let mut registry = registry();
    if registry.generation == generation {
        registry
            .grammars
            .entry(lang)
            .or_insert_with(|| loaded.clone());
    }
    loaded
}

pub fn generation() -> u64 {
    registry().generation
}

pub fn set_dir(dir: PathBuf) {
    let mut registry = registry();
    registry.dir = Some(dir);
    registry.reset();
}

pub fn set_languages(languages: Languages) {
    let mut registry = registry();
    registry.languages = languages;
    registry.reset();
}
