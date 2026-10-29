use std::collections::HashMap;
use std::env;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};

use crate::frame::Frame;
use crate::map_action::MapAction;
use crate::menu::MenuItem;
use crate::read_state::ReadState;
use crate::term::Term;
use crate::theme::Theme;
use crate::ui_state::UiState;
use crate::{key_feed, launch, render, requests};
use v_ide_editor::Error;
use v_ide_editor::document::Document;
use v_ide_editor::module::Modules;
use v_ide_editor::workspace::{DocId, WindowId, Workspace};
use v_ide_keys::{Keymap, TIMEOUT, Typeahead};
#[cfg(feature = "lua")]
use v_ide_lua::module::LuaModule;
use v_ide_markdown::module::MarkdownModule;
#[cfg(feature = "notebook")]
use v_ide_notebook::history::History;
#[cfg(feature = "notebook")]
use v_ide_notebook::module::NotebookModule;
#[cfg(feature = "vault")]
use v_ide_vault::module::VaultModule;
use v_ide_view::Depth;

pub struct App {
    pub workspace: Workspace,
    pub modules: Modules,
    pub ui: UiState,
    pub read_states: HashMap<DocId, ReadState>,
    pub quit: bool,
    pub theme: Theme,
    #[cfg(feature = "notebook")]
    pub history: History,
    pub nesting: usize,
    pub keymap: Keymap<MapAction>,
    pub typeahead: Typeahead,
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
        #[cfg(feature = "vault")]
        app.start_without_file();
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
                #[cfg(feature = "vault")]
                VaultModule::registration(),
                #[cfg(feature = "notebook")]
                NotebookModule::registration(),
                #[cfg(feature = "lua")]
                LuaModule::registration(),
            ]),
            ui: UiState::new(),
            read_states: HashMap::new(),
            quit: false,
            theme: Theme::default(),
            #[cfg(feature = "notebook")]
            history: History::new(),
            nesting: 0,
            keymap: Keymap::default(),
            typeahead: Typeahead::new(TIMEOUT),
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
        #[cfg(feature = "lua")]
        crate::lua_bridge::load_config(self);
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
            stale = event::poll(self.poll_wait())?;
            if stale
                && let Event::Key(key) = event::read()?
                && key.kind == KeyEventKind::Press
            {
                self.handle(key);
            }
            if self
                .typeahead
                .deadline()
                .is_some_and(|deadline| Instant::now() >= deadline)
            {
                self.timeout(Instant::now());
                stale = true;
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
        key_feed::feed(self, key);
    }

    pub fn timeout(&mut self, now: Instant) {
        key_feed::timeout(self, now);
    }

    fn poll_wait(&self) -> Duration {
        self.typeahead.deadline().map_or(TICK, |deadline| {
            TICK.min(deadline.saturating_duration_since(Instant::now()))
        })
    }

    pub fn external_editor(&self) -> Option<String> {
        self.code_editor
            .clone()
            .or_else(|| env::var("EDITOR").ok())
            .filter(|s| !s.trim().is_empty())
    }
}
