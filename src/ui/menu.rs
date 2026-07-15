use crate::markdown::wrap_mode::WrapMode;
use crate::ui::list_nav;

#[derive(Clone)]
pub struct MenuItem {
    pub key: char,
    pub label: String,
    pub action: String,
    pub children: Vec<MenuItem>,
}

impl MenuItem {
    fn new(key: char, label: &str, action: &str) -> Self {
        Self {
            key,
            label: label.to_string(),
            action: action.to_string(),
            children: Vec::new(),
        }
    }

    pub fn leaf(key: char, label: String, action: String) -> Self {
        Self {
            key,
            label,
            action,
            children: Vec::new(),
        }
    }

    fn submenu(key: char, label: &str, children: Vec<MenuItem>) -> Self {
        Self {
            key,
            label: format!("{label} \u{2192}"),
            action: String::new(),
            children,
        }
    }
}

pub struct Menu {
    pub title: String,
    pub items: Vec<MenuItem>,
    pub selected: usize,
    pub center: bool,
    stack: Vec<(String, Vec<MenuItem>, usize)>,
}

impl Menu {
    pub fn leader(extra: Vec<MenuItem>, center: bool, wrap: WrapMode, scan_hidden: bool) -> Self {
        let mut items = vec![
            MenuItem::new('f', "Find note", ":pick"),
            MenuItem::new(' ', "Buffers", ":buffers"),
            MenuItem::new('n', "New note", ":newprompt"),
            MenuItem::new('e', "Toggle read/edit", ":toggle-view"),
            MenuItem::new('t', "Toggle tree", ":tree"),
            MenuItem::new('b', "Backlinks", ":backlinks"),
            MenuItem::new('d', "Today's note", ":today"),
            MenuItem::new('w', "Write", ":w"),
            MenuItem::submenu('s', "Settings", settings(wrap, scan_hidden)),
            MenuItem::new('?', "Help", ":help"),
            MenuItem::new('q', "Quit", ":q"),
        ];
        items.extend(extra);
        Self::new(" menu ".to_string(), items, center)
    }

    pub fn empty_vault() -> Self {
        Self::new(
            " no notes here — get started ".to_string(),
            vec![
                MenuItem::new('n', "Create a note", ":newprompt"),
                MenuItem::new('d', "Create today's daily note", ":today"),
                MenuItem::new('?', "Help", ":help"),
                MenuItem::new('q', "Quit", ":q"),
            ],
            true,
        )
    }

    fn new(title: String, items: Vec<MenuItem>, center: bool) -> Self {
        Self {
            title,
            items,
            selected: 0,
            center,
            stack: Vec::new(),
        }
    }

    pub fn move_down(&mut self) {
        self.selected = list_nav::next(self.selected, self.items.len());
    }

    pub fn move_up(&mut self) {
        self.selected = list_nav::prev(self.selected);
    }

    pub fn index_of(&self, key: char) -> Option<usize> {
        self.items.iter().position(|item| item.key == key)
    }

    pub fn has_children(&self, idx: usize) -> bool {
        self.items.get(idx).is_some_and(|i| !i.children.is_empty())
    }

    pub fn action_of(&self, idx: usize) -> Option<String> {
        self.items.get(idx).map(|i| i.action.clone())
    }

    pub fn enter(&mut self, idx: usize) {
        let Some(item) = self.items.get(idx) else {
            return;
        };
        if item.children.is_empty() {
            return;
        }
        let label = item.label.trim_end_matches(" \u{2192}").to_lowercase();
        let children = item.children.clone();
        self.stack.push((
            std::mem::replace(&mut self.title, format!(" {label} ")),
            std::mem::replace(&mut self.items, children),
            self.selected,
        ));
        self.selected = 0;
    }

    pub fn back(&mut self) -> bool {
        match self.stack.pop() {
            Some((title, items, selected)) => {
                self.title = title;
                self.items = items;
                self.selected = selected;
                true
            }
            None => false,
        }
    }
}

fn settings(wrap: WrapMode, scan_hidden: bool) -> Vec<MenuItem> {
    vec![
        MenuItem::submenu('w', "Wrap", wrap_choices(wrap)),
        scan_hidden_choice(scan_hidden),
    ]
}

fn scan_hidden_choice(on: bool) -> MenuItem {
    let mark = if on { "\u{25cf} " } else { "  " };
    MenuItem::leaf(
        'h',
        format!("{mark}Scan hidden folders"),
        String::from(":scan-hidden toggle"),
    )
}

fn wrap_choices(wrap: WrapMode) -> Vec<MenuItem> {
    vec![
        wrap_choice('a', "All except table", "all", wrap == WrapMode::All),
        wrap_choice('t', "Text only", "text", wrap == WrapMode::Text),
        wrap_choice('n', "Nothing", "none", wrap == WrapMode::None),
    ]
}

fn wrap_choice(key: char, label: &str, mode: &str, current: bool) -> MenuItem {
    let mark = if current { "\u{25cf} " } else { "  " };
    MenuItem::leaf(key, format!("{mark}{label}"), format!(":wrap {mode}"))
}
