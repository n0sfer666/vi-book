use crate::vim::editor::Editor;
use crate::vim::key::Key;

#[derive(Default)]
pub struct SearchState {
    pub last: Option<String>,
    pub input: Option<String>,
}

pub fn begin(ed: &mut Editor) {
    ed.search.input = Some(String::new());
    ed.pending.reset();
}

pub fn handle_input(ed: &mut Editor, key: Key) {
    match key {
        Key::Char(c) => {
            if let Some(input) = ed.search.input.as_mut() {
                input.push(c);
            }
        }
        Key::Backspace => {
            if let Some(input) = ed.search.input.as_mut()
                && input.pop().is_none()
            {
                ed.search.input = None;
            }
        }
        Key::Enter => {
            if let Some(input) = ed.search.input.take()
                && !input.is_empty()
            {
                ed.search.last = Some(input);
                jump(ed, true);
            }
        }
        Key::Esc => ed.search.input = None,
        Key::Ctrl(_) | Key::Up | Key::Down | Key::Left | Key::Right => {}
    }
}

pub fn repeat(ed: &mut Editor, forward: bool) {
    jump(ed, forward);
    ed.pending.reset();
}

fn jump(ed: &mut Editor, forward: bool) {
    let Some(pattern) = ed.search.last.clone() else {
        return;
    };
    let chars: Vec<char> = ed.buffer.text().chars().collect();
    let pat: Vec<char> = pattern.chars().collect();
    let starts = occurrences(&chars, &pat);
    if starts.is_empty() {
        return;
    }
    let from = ed.cursor_index();
    let target = if forward {
        starts
            .iter()
            .find(|&&i| i > from)
            .or_else(|| starts.first())
    } else {
        starts
            .iter()
            .rev()
            .find(|&&i| i < from)
            .or_else(|| starts.last())
    };
    if let Some(&idx) = target {
        ed.cursor = ed.buffer.clamp_normal(ed.buffer.char_to_pos(idx));
    }
}

fn occurrences(chars: &[char], pat: &[char]) -> Vec<usize> {
    if pat.is_empty() || pat.len() > chars.len() {
        return Vec::new();
    }
    (0..=chars.len() - pat.len())
        .filter(|&i| chars[i..i + pat.len()] == *pat)
        .collect()
}
