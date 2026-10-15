use std::env;
use std::io::{self, Stdout, Write, stdout};
use std::panic;
use std::sync::Once;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::{Attribute, SetAttribute};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::{execute, queue};
use v_ide_view::{Depth, Grid, diff};

use crate::frame::Frame;
use crate::term_paint::paint;

static PANIC_HOOK: Once = Once::new();

pub struct Term {
    out: Stdout,
    prev: Grid,
    depth: Depth,
    active: bool,
}

impl Term {
    pub fn new(depth: Option<Depth>) -> Self {
        let detected = || {
            let colorterm = env::var("COLORTERM").ok();
            let term = env::var("TERM").ok();
            Depth::detect(colorterm.as_deref(), term.as_deref())
        };
        Self {
            out: stdout(),
            prev: Grid::new(0, 0),
            depth: depth.unwrap_or_else(detected),
            active: false,
        }
    }

    pub fn enter(&mut self) -> io::Result<()> {
        install_panic_hook();
        terminal::enable_raw_mode()?;
        execute!(self.out, EnterAlternateScreen, Hide)?;
        self.prev = Grid::new(0, 0);
        self.active = true;
        Ok(())
    }

    pub fn leave(&mut self) -> io::Result<()> {
        if !self.active {
            return Ok(());
        }
        self.active = false;
        restore()
    }

    pub fn size(&self) -> io::Result<(u16, u16)> {
        terminal::size()
    }

    pub fn draw(&mut self, frame: &Frame) -> io::Result<()> {
        let mut buf = Vec::new();
        paint(&mut buf, &diff(&self.prev, &frame.grid), self.depth)?;
        match frame.cursor {
            Some((x, y)) => queue!(buf, MoveTo(x, y), Show)?,
            None => queue!(buf, Hide)?,
        }
        self.out.write_all(&buf)?;
        self.out.flush()?;
        self.prev.clone_from(&frame.grid);
        Ok(())
    }
}

impl Drop for Term {
    fn drop(&mut self) {
        let _ = self.leave();
    }
}

fn restore() -> io::Result<()> {
    execute!(
        stdout(),
        SetAttribute(Attribute::Reset),
        Show,
        LeaveAlternateScreen
    )?;
    terminal::disable_raw_mode()
}

fn install_panic_hook() {
    PANIC_HOOK.call_once(|| {
        let hook = panic::take_hook();
        panic::set_hook(Box::new(move |info| {
            let _ = restore();
            hook(info);
        }));
    });
}
