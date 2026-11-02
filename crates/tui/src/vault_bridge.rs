use crate::app::App;
use crate::menu::Menu;
use crate::picker::Picker;
use v_ide_vault::module::VaultModule;

impl App {
    pub(crate) fn vault(&self) -> Option<&VaultModule> {
        self.modules.get::<VaultModule>()
    }

    pub(crate) fn vault_items(&mut self) -> Vec<String> {
        match self.vault().map(VaultModule::items) {
            Some(Ok(items)) => items,
            Some(Err(err)) => {
                self.ui.set_message(err);
                Vec::new()
            }
            None => Vec::new(),
        }
    }

    pub fn open_picker(&mut self) {
        let items = self.vault_items();
        self.ui.picker = Some(Picker::new(items, "e"));
    }

    pub(crate) fn start_without_file(&mut self) {
        let items = self.vault_items();
        if items.is_empty() {
            self.ui.menu = Some(Menu::empty_vault());
        } else {
            self.ui.picker = Some(Picker::new(items, "e"));
        }
    }
}
