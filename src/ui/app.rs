use std::collections::HashMap;
use std::env;

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;
use ratatui::layout::Rect;

use crate::io::error::Error;
use crate::lua::host::PluginHost;
use crate::markdown::rendered::Rendered;
use crate::notebook::date::Date;
use crate::notebook::history::History;
use crate::ui::menu::{Menu, MenuItem};
use crate::ui::panel::Panel;
use crate::ui::picker::Picker;
use crate::ui::session::Session;
use crate::ui::theme::Theme;
use crate::ui::view::View;
use crate::ui::{plugin, render, router};
use crate::vim::editor::Editor;
use crate::vim::key::Key;

pub struct App {
    pub editor: Editor,
    pub path: String,
    pub saved_text: String,
    pub top: usize,
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
    pub read_cache: Option<(String, usize, Rendered)>,
    pub menu: Option<Menu>,
    pub help: bool,
    pub leader: char,
    pub menu_center: bool,
    pub menu_extra: Vec<MenuItem>,
    pub insert_escape: Option<String>,
    pub others: Vec<crate::ui::pane::Pane>,
    pub active: usize,
    pub pending_ctrl_w: bool,
}

pub fn run() -> Result<(), Error> {
    let arg = env::args().nth(1);
    let (session, open) = Session::open(arg.as_deref())?;
    let mut app = App::new(Editor::new(""), String::new());
    app.session = session;
    app.today = crate::ui::clock::today();
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
    pub fn new(editor: Editor, path: String) -> Self {
        let saved_text = editor.text();
        Self {
            editor,
            path,
            saved_text,
            top: 0,
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
            menu: None,
            help: false,
            leader: ' ',
            menu_center: false,
            menu_extra: Vec::new(),
            insert_escape: Some(String::from("jj")),
            others: Vec::new(),
            active: 0,
            pending_ctrl_w: false,
        }
    }

    pub fn load(&mut self, rel: &str) {
        if self.editor.text() != self.saved_text {
            self.message = String::from("unsaved changes (write with :w or discard)");
            return;
        }
        match Editor::open(self.session.absolute(rel)) {
            Ok(mut editor) => {
                editor.set_insert_escape(self.insert_escape.clone());
                self.saved_text = editor.text();
                self.editor = editor;
                self.path = rel.to_string();
                self.top = 0;
                self.view = View::Read;
                self.read_cursor = (0, 0);
                self.read_cache = None;
                self.picker = None;
                self.menu = None;
            }
            Err(err) => self.message = format!("open failed: {err}"),
        }
    }

    pub fn open_rel(&mut self, rel: &str) {
        self.load(rel);
        if self.path == rel {
            self.history.visit(rel);
            plugin::fire(self, "note_open");
            self.saved_text = self.editor.text();
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
            if self.quit {
                return Ok(());
            }
        }
    }

    pub fn handle(&mut self, key: KeyEvent) {
        router::handle(self, key);
    }
}
