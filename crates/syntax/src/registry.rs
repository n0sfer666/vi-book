use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard, OnceLock, PoisonError};

use crate::{Grammar, LoadError, default_dir, load};

struct Registry {
    dir: Option<PathBuf>,
    generation: u64,
    grammars: HashMap<String, Option<Grammar>>,
}

static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();

fn registry() -> MutexGuard<'static, Registry> {
    REGISTRY
        .get_or_init(|| {
            Mutex::new(Registry {
                dir: default_dir(),
                generation: 0,
                grammars: HashMap::new(),
            })
        })
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
}

pub fn grammar(lang: &str) -> Option<Grammar> {
    let (dir, generation) = {
        let registry = registry();
        if let Some(cached) = registry.grammars.get(lang) {
            return cached.clone();
        }
        (registry.dir.clone(), registry.generation)
    };
    let loaded = match dir.as_deref().map(|dir| load(dir, lang)) {
        Some(Err(LoadError::BadName(_))) => return None,
        loaded => loaded.and_then(Result::ok),
    };
    let mut registry = registry();
    if registry.generation == generation {
        registry
            .grammars
            .entry(lang.to_string())
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
    registry.generation += 1;
    registry.grammars.clear();
}
