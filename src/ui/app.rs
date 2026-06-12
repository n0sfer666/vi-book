use std::env;

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;
use ratatui::layout::Rect;

use crate::io::error::Error;
use crate::notebook::date::Date;
use crate::notebook::history::History;
use crate::ui::panel::Panel;
use crate::ui::picker::Picker;
use crate::ui::session::Session;
use crate::ui::theme::Theme;
use crate::ui::view::View;
use crate::ui::{command_line, event_map, input, link_follow, read_view, render};
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
}

pub fn run() -> Result<(), Error> {
    let arg = env::args().nth(1);
    let (session, open) = Session::open(arg.as_deref())?;
    let mut app = App::new(Editor::new(""), String::new());
    app.session = session;
    app.today = crate::ui::clock::today();
    if let Some(rel) = open {
        app.open_rel(&rel);
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
        }
    }

    pub fn load(&mut self, rel: &str) {
        match Editor::open(self.session.absolute(rel)) {
            Ok(editor) => {
                self.saved_text = editor.text();
                self.editor = editor;
                self.path = rel.to_string();
                self.top = 0;
                self.view = View::Edit;
                self.picker = None;
            }
            Err(err) => self.message = format!("open failed: {err}"),
        }
    }

    pub fn open_rel(&mut self, rel: &str) {
        self.load(rel);
        if self.path == rel {
            self.history.visit(rel);
        }
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
        self.message.clear();
        if self.picker.is_some() {
            input::picker_key(self, key);
            return;
        }
        if self.command.is_some() {
            command_line::handle(self, key);
            return;
        }
        let mapped = event_map::to_key(key);
        if self.panel_focus && self.panel.is_some() {
            input::panel_key(self, mapped);
            return;
        }
        if self.editor.idle_normal() {
            match mapped {
                Some(Key::Char(':')) => {
                    self.command = Some(String::new());
                    return;
                }
                Some(Key::Ctrl('p')) => {
                    self.open_picker();
                    return;
                }
                Some(Key::Ctrl('w')) if self.panel.is_some() => {
                    self.panel_focus = true;
                    return;
                }
                Some(Key::Ctrl('o')) => {
                    if let Some(rel) = self.history.back() {
                        self.load(&rel);
                    }
                    return;
                }
                Some(Key::Ctrl('i')) => {
                    if let Some(rel) = self.history.forward() {
                        self.load(&rel);
                    }
                    return;
                }
                Some(Key::Enter) => {
                    link_follow::follow_at_cursor(self);
                    return;
                }
                _ => {}
            }
        }
        let width = read_view::column_width(self.text_area);
        if self.view == View::Read {
            read_view::handle_key(self, key, width);
            return;
        }
        if self.editor.idle_normal() && mapped == Some(Key::Ctrl('t')) {
            read_view::enter(self, width);
            return;
        }
        if let Some(key) = mapped {
            self.editor.handle_key(key);
        }
    }
}
