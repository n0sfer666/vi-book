use std::env;
use std::io;

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;
use ratatui::layout::Rect;

use crate::io::error::Error;
use crate::ui::view::View;
use crate::ui::{command_line, event_map, read_view, render};
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
}

pub fn run() -> Result<(), Error> {
    let path = env::args().nth(1).ok_or_else(|| {
        Error::Io(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: vi-book <file>",
        ))
    })?;
    let editor = Editor::open(&path)?;
    let mut terminal = ratatui::init();
    let result = App::new(editor, path).event_loop(&mut terminal);
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
        }
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
        if self.command.is_some() {
            command_line::handle(self, key);
            return;
        }
        let mapped = event_map::to_key(key);
        if self.editor.idle_normal() && mapped == Some(Key::Char(':')) {
            self.command = Some(String::new());
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
        if let Some(key) = mapped {
            self.editor.handle_key(key);
        }
    }
}
