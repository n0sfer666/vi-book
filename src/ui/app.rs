use std::env;
use std::io;

use crossterm::event::{self, Event, KeyEvent, KeyEventKind};
use ratatui::DefaultTerminal;

use crate::io::error::Error;
use crate::ui::{command_line, event_map, render};
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
        if self.editor.idle_normal() && event_map::to_key(key) == Some(Key::Char(':')) {
            self.command = Some(String::new());
            return;
        }
        if let Some(key) = event_map::to_key(key) {
            self.editor.handle_key(key);
        }
    }
}
