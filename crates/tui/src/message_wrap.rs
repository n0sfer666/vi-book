use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

pub(crate) struct WrappedRow {
    pub(crate) start: usize,
    pub(crate) text: String,
}

pub(crate) fn normalize(text: &str) -> String {
    let mut out = String::new();
    for c in text.chars() {
        match c {
            '\n' | '\t' => out.push(c),
            '\0'..='\u{1f}' => {
                out.push('^');
                out.push(char::from(u8::try_from(c).unwrap_or(0) + b'@'));
            }
            '\u{7f}' => out.push_str("^?"),
            c if c.is_control() => out.push_str(&c.escape_default().to_string()),
            c => out.push(c),
        }
    }
    out
}

pub(crate) fn wrap(text: &str, width: u16) -> Vec<WrappedRow> {
    let mut builder = Builder {
        rows: Vec::new(),
        row: String::new(),
        start: 0,
        columns: 0,
        width: usize::from(width.max(1)),
    };
    for (at, grapheme) in text.grapheme_indices(true) {
        match grapheme {
            "\n" => builder.finish(at + 1),
            "\t" => {
                let spaces = 8 - builder.columns % 8;
                for _ in 0..spaces {
                    builder.add(" ", 1, at);
                }
            }
            g if g.width() > builder.width => builder.add("�", 1, at),
            g => builder.add(g, g.width(), at),
        }
    }
    builder.finish(text.len());
    builder.rows
}

struct Builder {
    rows: Vec<WrappedRow>,
    row: String,
    start: usize,
    columns: usize,
    width: usize,
}

impl Builder {
    fn add(&mut self, text: &str, width: usize, at: usize) {
        if self.columns + width > self.width {
            self.finish(at);
        }
        self.row.push_str(text);
        self.columns += width;
    }

    fn finish(&mut self, next: usize) {
        self.rows.push(WrappedRow {
            start: self.start,
            text: std::mem::take(&mut self.row),
        });
        self.columns = 0;
        self.start = next;
    }
}
