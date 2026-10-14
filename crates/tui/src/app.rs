use std::collections::HashMap;
use std::env;

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::menu::{Menu, MenuItem};
use crate::picker::Picker;
use crate::read_state::ReadState;
use crate::theme::Theme;
use crate::ui_state::UiState;
use crate::{launch, plugin, render, requests, router};
use v_ide_editor::Error;
use v_ide_editor::document::Document;
use v_ide_editor::module::Modules;
use v_ide_editor::workspace::{DocId, WindowId, Workspace};
use v_ide_keys::key::Key;
use v_ide_lua::host::PluginHost;
use v_ide_markdown::wrap_mode::WrapMode;
use v_ide_notebook::history::History;
use v_ide_notebook::module::NotebookModule;
use v_ide_vault::module::VaultModule;

pub struct App {
    pub workspace: Workspace,
    pub modules: Modules,
    pub ui: UiState,
    pub read_states: HashMap<DocId, ReadState>,
    pub quit: bool,
    pub theme: Theme,
    pub history: History,
    pub host: Option<PluginHost>,
    pub firing: bool,
    pub nesting: usize,
    pub normal_maps: HashMap<Key, String>,
    pub wrap: WrapMode,
    pub leader: char,
    pub menu_center: bool,
    pub menu_extra: Vec<MenuItem>,
    pub insert_escape: Option<String>,
    pub window_states: HashMap<WindowId, ReadState>,
    pub code_editor: Option<String>,
    pub external_request: Option<std::path::PathBuf>,
    pub url_request: Option<String>,
    pub last_clip: String,
}

pub fn run() -> Result<(), Error> {
    let arg = env::args().nth(1);
    let (cwd, open) = launch::target(arg.as_deref())?;
    let mut app = App::new(Document::new(""));
    app.workspace.set_cwd(cwd);
    requests::startup(&mut app);
    app.load_config();
    app.install_keymaps();
    if let Some(rel) = open {
        app.open_rel(&rel);
    } else if app.vault().is_none_or(VaultModule::is_empty) {
        app.ui.menu = Some(Menu::empty_vault());
    } else {
        app.open_picker();
    }
    let mut terminal = ratatui::init();
    let result = app.event_loop(&mut terminal);
    ratatui::restore();
    result
}

impl App {
    pub fn new(doc: Document) -> Self {
        Self {
            workspace: Workspace::new(doc),
            modules: Modules::new(vec![
                VaultModule::registration(),
                NotebookModule::registration(),
            ]),
            ui: UiState::new(),
            read_states: HashMap::new(),
            quit: false,
            theme: Theme::default(),
            history: History::new(),
            host: None,
            firing: false,
            nesting: 0,
            normal_maps: HashMap::new(),
            wrap: WrapMode::All,
            leader: ' ',
            menu_center: false,
            menu_extra: Vec::new(),
            insert_escape: Some(String::from("jj")),
            window_states: HashMap::new(),
            code_editor: None,
            external_request: None,
            url_request: None,
            last_clip: String::new(),
        }
    }

    pub fn load_config(&mut self) {
        plugin::load_config(self);
    }

    pub fn install_keymaps(&mut self) {
        plugin::install_keymaps(self);
    }

    pub fn open_picker(&mut self) {
        self.ui.picker = Some(Picker::new(self.vault_items(), "e"));
    }

    fn event_loop(&mut self, terminal: &mut DefaultTerminal) -> Result<(), Error> {
        loop {
            terminal.draw(|frame| render::draw(frame, self))?;
            if let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.handle(key);
            }
            if let Some(path) = self.external_request.take() {
                if let Some(editor) = self.external_editor() {
                    let _ = crate::external::run(terminal, &editor, &path);
                } else {
                    self.ui.message =
                        String::from("no editor set (vibook.opt.code_editor or $EDITOR)");
                }
            }
            if let Some(url) = self.url_request.take() {
                crate::external::open_url(&url);
            }
            if self.quit {
                return Ok(());
            }
        }
    }

    pub fn handle(&mut self, key: KeyEvent) {
        router::handle(self, key);
    }

    pub fn external_editor(&self) -> Option<String> {
        self.code_editor
            .clone()
            .or_else(|| env::var("EDITOR").ok())
            .filter(|s| !s.trim().is_empty())
    }
}
