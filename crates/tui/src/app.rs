use std::collections::HashMap;
use std::env;
use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};

use crate::frame::Frame;
use crate::menu::{Menu, MenuItem};
use crate::picker::Picker;
use crate::read_state::ReadState;
use crate::term::Term;
use crate::theme::Theme;
use crate::ui_state::UiState;
use crate::{launch, lua_bridge, render, requests, router};
use v_ide_editor::Error;
use v_ide_editor::document::Document;
use v_ide_editor::module::Modules;
use v_ide_editor::workspace::{DocId, WindowId, Workspace};
use v_ide_keys::key::Key;
use v_ide_lua::module::LuaModule;
use v_ide_markdown::module::MarkdownModule;
use v_ide_notebook::history::History;
use v_ide_notebook::module::NotebookModule;
use v_ide_vault::module::VaultModule;
use v_ide_view::Depth;

pub struct App {
    pub workspace: Workspace,
    pub modules: Modules,
    pub ui: UiState,
    pub read_states: HashMap<DocId, ReadState>,
    pub quit: bool,
    pub theme: Theme,
    pub history: History,
    pub nesting: usize,
    pub normal_maps: HashMap<Key, String>,
    pub leader: char,
    pub menu_center: bool,
    pub menu_extra: Vec<MenuItem>,
    pub insert_escape: Option<String>,
    pub window_states: HashMap<WindowId, ReadState>,
    pub code_editor: Option<String>,
    pub external_request: Option<std::path::PathBuf>,
    pub url_request: Option<String>,
    pub last_clip: String,
    pub color_depth: Option<Depth>,
}

const TICK: Duration = Duration::from_millis(100);

pub fn run() -> Result<(), Error> {
    let arg = env::args().nth(1);
    let (cwd, open) = launch::target(arg.as_deref())?;
    let mut app = App::new(Document::new(""));
    app.workspace.set_cwd(cwd);
    requests::startup(&mut app);
    app.load_config();
    if let Some(rel) = open {
        app.open_rel(&rel);
    } else {
        let items = app.vault_items();
        if items.is_empty() {
            app.ui.menu = Some(Menu::empty_vault());
        } else {
            app.ui.picker = Some(Picker::new(items, "e"));
        }
    }
    let mut term = Term::new(app.color_depth);
    term.enter()?;
    app.event_loop(&mut term)
}

impl App {
    pub fn new(doc: Document) -> Self {
        Self {
            workspace: Workspace::new(doc),
            modules: Modules::new(vec![
                MarkdownModule::registration(),
                VaultModule::registration(),
                NotebookModule::registration(),
                LuaModule::registration(),
            ]),
            ui: UiState::new(),
            read_states: HashMap::new(),
            quit: false,
            theme: Theme::default(),
            history: History::new(),
            nesting: 0,
            normal_maps: HashMap::new(),
            leader: ' ',
            menu_center: false,
            menu_extra: Vec::new(),
            insert_escape: Some(String::from("jj")),
            window_states: HashMap::new(),
            code_editor: None,
            external_request: None,
            url_request: None,
            last_clip: String::new(),
            color_depth: None,
        }
    }

    pub fn load_config(&mut self) {
        lua_bridge::load_config(self);
    }

    pub fn open_picker(&mut self) {
        let items = self.vault_items();
        self.ui.picker = Some(Picker::new(items, "e"));
    }

    fn event_loop(&mut self, term: &mut Term) -> Result<(), Error> {
        let mut stale = true;
        loop {
            if stale {
                let (w, h) = term.size()?;
                let mut frame = Frame::new(w, h);
                render::draw(&mut frame, self);
                term.draw(&frame)?;
            }
            stale = event::poll(TICK)?;
            if stale
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.handle(key);
            }
            stale |= requests::tick(self);
            if let Some(path) = self.external_request.take() {
                stale = true;
                if let Some(editor) = self.external_editor() {
                    let _ = crate::external::run(term, &editor, &path);
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
