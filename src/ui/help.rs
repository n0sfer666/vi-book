pub const MANUAL: &str = include_str!("../../docs/manual.md");
pub const DEVELOPERS: &str = include_str!("../../docs/developers.md");

pub const BINDINGS: &[(&str, &str)] = &[
    ("<Space>", "open the menu (leader)"),
    ("F1 / ?", "this help"),
    ("i", "edit the current note"),
    ("<Esc> / jj", "leave edit mode"),
    ("<C-t>", "toggle read / edit"),
    ("h j k l / arrows", "move cursor"),
    ("w / b / e", "word forward / back / end"),
    ("W / B", "next / previous block (heading, paragraph, item)"),
    ("gg / G", "top / bottom"),
    ("<C-p>", "find note (fuzzy)"),
    ("<CR>", "follow [[link]] under cursor"),
    ("<C-w>", "focus side panel"),
    ("<C-o> / <C-i>", "jump back / forward"),
    (":w :q :wq", "write / quit"),
    (":tree :backlinks", "toggle panels"),
    (":today :new <name>", "daily note / create note"),
];
