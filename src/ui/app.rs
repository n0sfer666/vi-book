use std::env;

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;
use ratatui::layout::Rect;

use crate::io::error::Error;
use crate::ui::picker::Picker;
use crate::ui::session::Session;
use crate::ui::theme::Theme;
use crate::ui::view::View;
use crate::ui::{command_line, event_map, link_follow, read_view, render};
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
}

pub fn run() -> Result<(), Error> {
    let arg = env::args().nth(1);
    let (session, open) = Session::open(arg.as_deref())?;
    let mut app = App::new(Editor::new(""), String::new());
    app.session = session;
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
        }
    }

    pub fn open_rel(&mut self, rel: &str) {
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
            self.handle_picker(key);
            return;
        }
        if self.command.is_some() {
            command_line::handle(self, key);
            return;
        }
        let mapped = event_map::to_key(key);
        if self.editor.idle_normal() && mapped == Some(Key::Char(':')) {
            self.command = Some(String::new());
            return;
        }
        if self.editor.idle_normal() && mapped == Some(Key::Ctrl('p')) {
            self.open_picker();
            return;
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
        if self.editor.idle_normal() && mapped == Some(Key::Enter) {
            link_follow::follow_at_cursor(self);
            return;
        }
        if let Some(key) = mapped {
            self.editor.handle_key(key);
        }
    }

    fn handle_picker(&mut self, key: KeyEvent) {
        let Some(picker) = self.picker.as_mut() else {
            return;
        };
        match event_map::to_key(key) {
            Some(Key::Esc) => self.picker = None,
            Some(Key::Enter) => {
                if let Some(rel) = picker.current() {
                    self.open_rel(&rel);
                } else {
                    self.picker = None;
                }
            }
            Some(Key::Backspace) => picker.pop(),
            Some(Key::Ctrl('n')) => picker.move_down(),
            Some(Key::Ctrl('p')) => picker.move_up(),
            Some(Key::Char(c)) => picker.push(c),
            _ => {}
        }
    }
}
