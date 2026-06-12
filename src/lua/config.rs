use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keymap {
    pub mode: String,
    pub lhs: String,
    pub rhs: String,
}

#[derive(Default)]
pub struct Bridge {
    pub buf_text: String,
    pub buf_cursor: usize,
    pub vault_notes: Vec<String>,
    pub vault_tags: Vec<String>,
    pub backlinks: HashMap<String, Vec<String>>,
    pub creates: Vec<String>,
    pub opens: Vec<String>,
}

pub type Shared = Rc<RefCell<Bridge>>;

pub const PRELUDE: &str = r#"
vibook = vibook or {}
vibook.opt = {}
vibook.__commands = {}
vibook.__handlers = {}
vibook.__keymaps = {}
function vibook.command(name, fn) vibook.__commands[name] = fn end
function vibook.on(event, fn)
  vibook.__handlers[event] = vibook.__handlers[event] or {}
  table.insert(vibook.__handlers[event], fn)
end
function vibook.keymap(mode, lhs, rhs)
  table.insert(vibook.__keymaps, { mode = mode, lhs = lhs, rhs = rhs })
end
"#;
