use std::collections::HashMap;
use std::env;

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;
use ratatui::layout::Rect;

use crate::menu::{Menu, MenuItem};
use crate::panel::Panel;
use crate::picker::Picker;
use crate::read_state::ReadState;
use crate::session::Session;
use crate::theme::Theme;
use crate::view::View;
use crate::{plugin, render, router};
use v_ide_editor::Error;
use v_ide_editor::document::Document;
use v_ide_editor::workspace::{DocId, WindowId, Workspace};
use v_ide_keys::key::Key;
use v_ide_lua::host::PluginHost;
use v_ide_markdown::rendered::Rendered;
use v_ide_markdown::wrap_mode::WrapMode;
use v_ide_notebook::date::Date;
use v_ide_notebook::history::History;

pub struct App {
    pub workspace: Workspace,
    pub read_states: HashMap<DocId, ReadState>,
    pub top: usize,
    pub left: usize,
    pub command: Option<String>,
    pub message: String,
    pub quit: bool,
    pub view: View,
    pub read_cursor: (usize, usize),
    pub read_pending_g: bool,
    pub text_area: Rect,
    pub session: Session,
    pub picker: Option<Picker>,
    pub theme: Theme,
    pub history: History,
    pub today: Date,
    pub panel: Option<Panel>,
    pub panel_focus: bool,
    pub host: Option<PluginHost>,
    pub firing: bool,
    pub normal_maps: HashMap<Key, String>,
    pub read_cache: Option<(String, usize, WrapMode, Rendered)>,
    pub wrap: WrapMode,
    pub menu: Option<Menu>,
    pub help: bool,
    pub leader: char,
    pub menu_center: bool,
    pub menu_extra: Vec<MenuItem>,
    pub insert_escape: Option<String>,
    pub window_states: HashMap<WindowId, ReadState>,
    pub buffer_list: Option<crate::buffer_list::BufferList>,
    pub pending_ctrl_w: bool,
    pub code_editor: Option<String>,
    pub external_request: Option<std::path::PathBuf>,
    pub url_request: Option<String>,
    pub read_select: Option<(v_ide_engine::mode::Mode, (usize, usize))>,
    pub last_clip: String,
}

pub fn run() -> Result<(), Error> {
    let arg = env::args().nth(1);
    let (session, open) = Session::open(arg.as_deref())?;
    let mut app = App::new(Document::new(""));
    app.session = session;
    app.today = crate::clock::today();
    app.load_config();
    app.install_keymaps();
    if let Some(rel) = open {
        app.open_rel(&rel);
    } else if app.session.is_empty() {
        app.menu = Some(Menu::empty_vault());
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
            read_states: HashMap::new(),
            top: 0,
            left: 0,
            command: None,
            message: String::new(),
            quit: false,
            view: View::default(),
            read_cursor: (0, 0),
            read_pending_g: false,
            text_area: Rect::new(0, 0, 80, 24),
            session: Session::in_memory(),
            picker: None,
            theme: Theme::default(),
            history: History::new(),
            today: Date::new(1970, 1, 1),
            panel: None,
            panel_focus: false,
            host: None,
            firing: false,
            normal_maps: HashMap::new(),
            read_cache: None,
            wrap: WrapMode::All,
            menu: None,
            help: false,
            leader: ' ',
            menu_center: false,
            menu_extra: Vec::new(),
            insert_escape: Some(String::from("jj")),
            window_states: HashMap::new(),
            buffer_list: None,
            pending_ctrl_w: false,
            code_editor: None,
            external_request: None,
            url_request: None,
            read_select: None,
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
        self.picker = Some(Picker::new(self.session.items()));
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
                    self.message =
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
