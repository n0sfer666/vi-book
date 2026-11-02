use std::cell::Cell;
use std::rc::{Rc, Weak};

use crate::Position;

#[derive(Debug)]
pub struct TrackedPosition(Rc<Cell<Position>>);

impl TrackedPosition {
    pub fn get(&self) -> Position {
        self.0.get()
    }
}

#[derive(Default)]
pub(crate) struct Positions(Vec<Weak<Cell<Position>>>);

impl Positions {
    pub(crate) fn track(&mut self, position: Position) -> TrackedPosition {
        self.0.retain(|position| position.strong_count() > 0);
        let position = Rc::new(Cell::new(position));
        self.0.push(Rc::downgrade(&position));
        TrackedPosition(position)
    }

    fn update(&mut self, mut update: impl FnMut(Position) -> Position) {
        self.0.retain(|position| {
            let Some(position) = position.upgrade() else {
                return false;
            };
            position.set(update(position.get()));
            true
        });
    }

    pub(crate) fn split(&mut self, at: Position, inserted: usize) {
        if inserted == 0 {
            return;
        }
        self.update(|mut position| {
            if position.line > at.line {
                position.line += inserted;
            }
            position
        });
    }

    pub(crate) fn remove(&mut self, start: Position, end: Position) {
        if start.line == end.line {
            return;
        }
        self.update(|mut position| {
            if position.line > end.line {
                position.line -= end.line - start.line;
            } else if position.line > start.line {
                position.col = if position.line == end.line && position.col >= end.col {
                    start.col + position.col - end.col
                } else {
                    0
                };
                position.line = start.line;
            }
            position
        });
    }

    pub(crate) fn lines(&mut self, at: usize, removed: usize, inserted: usize) {
        self.update(|mut position| {
            if removed > 0 && (at..at + removed).contains(&position.line) {
                position.line = at.saturating_sub(1);
                position.col = 0;
            } else if position.line >= at + removed {
                position.line = position.line - removed + inserted;
            }
            position
        });
    }
}
