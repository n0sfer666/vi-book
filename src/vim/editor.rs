use std::path::Path;

use crate::core::position::Position;
use crate::io::error::Error;
use crate::vim::key::{Key, parse_keys};
use crate::vim::mode::Mode;

pub struct Editor;

impl Editor {
    pub fn new(_text: &str) -> Self {
        todo!()
    }

    pub fn open(_path: impl AsRef<Path>) -> Result<Self, Error> {
        todo!()
    }

    pub fn save(&mut self) -> Result<(), Error> {
        todo!()
    }

    pub fn text(&self) -> String {
        todo!()
    }

    pub fn cursor(&self) -> Position {
        todo!()
    }

    pub fn set_cursor(&mut self, _pos: Position) {
        todo!()
    }

    pub fn mode(&self) -> Mode {
        todo!()
    }

    pub fn handle_key(&mut self, _key: Key) {
        todo!()
    }

    pub fn handle_keys(&mut self, notation: &str) {
        for key in parse_keys(notation) {
            self.handle_key(key);
        }
    }
}
