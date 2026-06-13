use crate::ui::list_nav;

#[derive(Clone)]
pub struct MenuItem {
    pub key: char,
    pub label: String,
    pub action: String,
}

impl MenuItem {
    fn new(key: char, label: &str, action: &str) -> Self {
        Self {
            key,
            label: label.to_string(),
            action: action.to_string(),
        }
    }
}

pub struct Menu {
    pub title: String,
    pub items: Vec<MenuItem>,
    pub selected: usize,
    pub center: bool,
}

impl Menu {
    pub fn leader(extra: Vec<MenuItem>, center: bool) -> Self {
        let mut items = vec![
            MenuItem::new('f', "Find note", ":pick"),
            MenuItem::new('n', "New note", ":newprompt"),
            MenuItem::new('e', "Toggle read/edit", ":toggle-view"),
            MenuItem::new('t', "Toggle tree", ":tree"),
            MenuItem::new('b', "Backlinks", ":backlinks"),
            MenuItem::new('d', "Today's note", ":today"),
            MenuItem::new('w', "Write", ":w"),
            MenuItem::new('?', "Help", ":help"),
            MenuItem::new('q', "Quit", ":q"),
        ];
        items.extend(extra);
        Self {
            title: " menu ".to_string(),
            items,
            selected: 0,
            center,
        }
    }

    pub fn empty_vault() -> Self {
        Self {
            title: " no notes here — get started ".to_string(),
            items: vec![
                MenuItem::new('n', "Create a note", ":newprompt"),
                MenuItem::new('d', "Create today's daily note", ":today"),
                MenuItem::new('?', "Help", ":help"),
                MenuItem::new('q', "Quit", ":q"),
            ],
            selected: 0,
            center: true,
        }
    }

    pub fn move_down(&mut self) {
        self.selected = list_nav::next(self.selected, self.items.len());
    }

    pub fn move_up(&mut self) {
        self.selected = list_nav::prev(self.selected);
    }

    pub fn current(&self) -> Option<&MenuItem> {
        self.items.get(self.selected)
    }

    pub fn by_key(&self, key: char) -> Option<&MenuItem> {
        self.items.iter().find(|i| i.key == key)
    }
}
