use std::cell::RefCell;
use std::rc::Rc;

use v_ide_editor::module::VaultQuery;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keymap {
    pub mode: Vec<String>,
    pub lhs: String,
    pub rhs: String,
    pub remap: bool,
    pub nowait: bool,
    pub unique: bool,
}

#[derive(Default)]
pub struct Bridge {
    pub options: crate::options::Options,
    pub buf_text: String,
    pub buf_cursor: usize,
    pub buffer_pending: bool,
    pub vault: Option<Rc<dyn VaultQuery>>,
    pub creates: Vec<String>,
    pub opens: Vec<String>,
    pub external_opens: Vec<String>,
}

pub type Shared = Rc<RefCell<Bridge>>;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ColorSpec {
    pub name: String,
    pub fg: Option<String>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub dim: bool,
}

pub const PRELUDE: &str = r#"
vibook = vibook or {}
vibook.opt = { read_number = false }
vibook.__commands = {}
vibook.__handlers = {}
vibook.__keymaps = {}
vibook.__keymap_errors = {}
vibook.__menu = {}
vibook.__colors = {}
function vibook.menu_item(key, label, rhs)
  table.insert(vibook.__menu, { key = key, label = label, rhs = rhs })
end
function vibook.color(name, spec)
  table.insert(vibook.__colors, { name = name, spec = spec })
end
function vibook.command(name, fn) vibook.__commands[name] = fn end
function vibook.on(event, fn)
  vibook.__handlers[event] = vibook.__handlers[event] or {}
  table.insert(vibook.__handlers[event], fn)
end
local function keymap_error(message)
  table.insert(vibook.__keymap_errors, "keymap: " .. message)
end
function vibook.keymap(mode, lhs, rhs, opts)
  if (type(mode) ~= "string" and type(mode) ~= "table") or type(lhs) ~= "string" or type(rhs) ~= "string" then
    keymap_error("mode must be a string or list; lhs and rhs must be strings")
    return
  end
  if type(mode) == "table" then
    for index, value in pairs(mode) do
      if type(index) ~= "number" or index % 1 ~= 0 or index < 1 or index > #mode or type(value) ~= "string" then
        keymap_error("mode must be a string or list of strings")
        return
      end
    end
    local snapshot = {}
    for index, value in ipairs(mode) do snapshot[index] = value end
    mode = snapshot
  end
  if opts ~= nil and type(opts) ~= "table" then
    keymap_error("opts must be a table")
    return
  end
  opts = opts or {}
  local supported = { remap = true, nowait = true, unique = true, silent = true }
  for name, value in pairs(opts) do
    if not supported[name] then
      keymap_error("unsupported option " .. tostring(name))
      return
    end
    if type(value) ~= "boolean" then
      keymap_error(tostring(name) .. " must be a boolean")
      return
    end
  end
  table.insert(vibook.__keymaps, {
    mode = mode, lhs = lhs, rhs = rhs,
    remap = opts.remap or false, nowait = opts.nowait or false, unique = opts.unique or false,
  })
end
"#;
