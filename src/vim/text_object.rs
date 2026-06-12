use crate::vim::editor::Editor;
use crate::vim::motion::OpTarget;
use crate::vim::word;

pub fn target(ed: &Editor, object: char, around: bool) -> Option<OpTarget> {
    if around && object != 'w' {
        return None;
    }
    match object {
        'w' => word_object(ed, around),
        '(' => parens(ed),
        '"' => quotes(ed),
        'p' => paragraph(ed),
        _ => None,
    }
}

fn word_object(ed: &Editor, around: bool) -> Option<OpTarget> {
    let buffer = &ed.buffer;
    let idx = ed.cursor_index();
    let (mut start, mut end) = word::word_run(buffer, idx)?;
    if around {
        let is_line_blank = |i: usize| {
            buffer
                .char_at(i)
                .is_some_and(|c| c.is_whitespace() && c != '\n')
        };
        let mut extended = false;
        while is_line_blank(end + 1) {
            end += 1;
            extended = true;
        }
        if !extended {
            while start > 0 && is_line_blank(start - 1) {
                start -= 1;
            }
        }
    }
    Some(OpTarget::Chars(start..end + 1))
}

fn parens(ed: &Editor) -> Option<OpTarget> {
    let buffer = &ed.buffer;
    let idx = ed.cursor_index();
    let open = if buffer.char_at(idx) == Some('(') {
        idx
    } else {
        let mut depth = 0;
        let mut found = None;
        let mut i = idx;
        while i > 0 {
            i -= 1;
            match buffer.char_at(i) {
                Some(')') => depth += 1,
                Some('(') if depth == 0 => {
                    found = Some(i);
                    break;
                }
                Some('(') => depth -= 1,
                _ => {}
            }
        }
        found?
    };
    let mut depth = 0;
    let mut close = None;
    for i in open + 1..buffer.len_chars() {
        match buffer.char_at(i) {
            Some('(') => depth += 1,
            Some(')') if depth == 0 => {
                close = Some(i);
                break;
            }
            Some(')') => depth -= 1,
            _ => {}
        }
    }
    Some(OpTarget::Chars(open + 1..close?))
}

fn quotes(ed: &Editor) -> Option<OpTarget> {
    let buffer = &ed.buffer;
    let line = ed.cursor.line;
    let start = buffer.line_start(line);
    let cols: Vec<usize> = (0..buffer.line_len(line))
        .filter(|&col| buffer.char_at(start + col) == Some('"'))
        .collect();
    for pair in cols.chunks(2) {
        if let [open, close] = *pair
            && ed.cursor.col <= close
        {
            return Some(OpTarget::Chars(start + open + 1..start + close));
        }
    }
    None
}

fn paragraph(ed: &Editor) -> Option<OpTarget> {
    let buffer = &ed.buffer;
    let current = ed.cursor.line;
    let blank = buffer.is_blank_line(current);
    let mut first = current;
    while first > 0 && buffer.is_blank_line(first - 1) == blank {
        first -= 1;
    }
    let mut last = current;
    while last + 1 < buffer.line_count() && buffer.is_blank_line(last + 1) == blank {
        last += 1;
    }
    Some(OpTarget::Lines { first, last })
}
