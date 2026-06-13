use crate::core::buffer::Buffer;
use crate::markdown::rendered::Rendered;

type Pos = (usize, usize);

fn lines(rendered: &Rendered) -> Vec<String> {
    rendered.lines.iter().map(|l| l.text()).collect()
}

fn pos_to_idx(lines: &[String], pos: Pos) -> usize {
    let mut idx = 0;
    for line in lines.iter().take(pos.0) {
        idx += line.chars().count() + 1;
    }
    idx + pos.1
}

fn idx_to_pos(lines: &[String], idx: usize) -> Pos {
    let mut rest = idx;
    for (line, text) in lines.iter().enumerate() {
        let len = text.chars().count();
        if rest <= len {
            return (line, rest);
        }
        rest -= len + 1;
    }
    let last = lines.len().saturating_sub(1);
    (last, lines.get(last).map_or(0, |l| l.chars().count()))
}

fn first_non_blank(line: &str) -> usize {
    line.chars()
        .position(|c| !c.is_whitespace())
        .unwrap_or_else(|| line.chars().count())
}

pub fn word_forward(rendered: &Rendered, pos: Pos) -> Pos {
    word_step(rendered, pos, crate::vim::word::next_word_start)
}

pub fn word_back(rendered: &Rendered, pos: Pos) -> Pos {
    word_step(rendered, pos, crate::vim::word::prev_word_start)
}

pub fn word_end(rendered: &Rendered, pos: Pos) -> Pos {
    word_step(rendered, pos, crate::vim::word::next_word_end)
}

fn word_step(rendered: &Rendered, pos: Pos, step: fn(&Buffer, usize) -> usize) -> Pos {
    let lines = lines(rendered);
    let text = lines.join("\n");
    let buffer = Buffer::new(&text);
    let idx = pos_to_idx(&lines, pos).min(buffer.len_chars());
    idx_to_pos(&lines, step(&buffer, idx))
}

pub fn element_forward(rendered: &Rendered, pos: Pos) -> Pos {
    let lines = lines(rendered);
    for l in (pos.0 + 1)..lines.len() {
        if is_element_start(&lines, l) {
            return (l, first_non_blank(&lines[l]));
        }
    }
    let last = lines.len().saturating_sub(1);
    (
        last,
        first_non_blank(lines.get(last).map_or("", String::as_str)),
    )
}

pub fn element_back(rendered: &Rendered, pos: Pos) -> Pos {
    let lines = lines(rendered);
    for l in (0..pos.0).rev() {
        if is_element_start(&lines, l) {
            return (l, first_non_blank(&lines[l]));
        }
    }
    (0, 0)
}

fn is_element_start(lines: &[String], l: usize) -> bool {
    let line = &lines[l];
    if line.trim().is_empty() {
        return false;
    }
    l == 0 || lines[l - 1].trim().is_empty() || is_heading(line)
}

fn is_heading(line: &str) -> bool {
    let t = line.trim_start();
    t.starts_with('#') || t.starts_with("- ") || t.starts_with("* ")
}
