use crate::app::App;
use v_ide_vault::module::VaultModule;

impl App {
    pub(crate) fn vault(&self) -> Option<&VaultModule> {
        self.modules.get::<VaultModule>()
    }

    pub(crate) fn vault_items(&self) -> Vec<String> {
        self.vault().map(VaultModule::items).unwrap_or_default()
    }
}
