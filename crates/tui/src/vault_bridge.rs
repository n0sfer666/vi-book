use crate::app::App;
use v_ide_vault::module::VaultModule;

impl App {
    pub(crate) fn vault(&self) -> Option<&VaultModule> {
        self.modules.get::<VaultModule>()
    }

    pub(crate) fn vault_items(&mut self) -> Vec<String> {
        match self.vault().map(VaultModule::items) {
            Some(Ok(items)) => items,
            Some(Err(err)) => {
                self.ui.message = err;
                Vec::new()
            }
            None => Vec::new(),
        }
    }
}
