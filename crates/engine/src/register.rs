use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct RegisterContent {
    pub text: String,
    pub linewise: bool,
}

impl RegisterContent {
    pub fn charwise(text: String) -> Self {
        Self {
            text,
            linewise: false,
        }
    }

    pub fn linewise(text: String) -> Self {
        Self {
            text,
            linewise: true,
        }
    }
}

#[derive(Default)]
pub struct Registers {
    unnamed: RegisterContent,
    named: HashMap<char, RegisterContent>,
    yank_version: u64,
}

impl Registers {
    pub fn write(&mut self, name: Option<char>, content: RegisterContent) {
        if let Some(name) = name {
            self.named.insert(name, content.clone());
        }
        self.unnamed = content;
    }

    pub fn note_yank(&mut self) {
        self.yank_version += 1;
    }

    pub fn yank_version(&self) -> u64 {
        self.yank_version
    }

    pub fn read(&self, name: Option<char>) -> Option<&RegisterContent> {
        match name {
            Some(name) => self.named.get(&name),
            None => Some(&self.unnamed),
        }
    }
}
