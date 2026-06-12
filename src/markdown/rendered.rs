use crate::markdown::style::Span;

#[derive(Debug, Clone, Default)]
pub struct RenderedLine {
    pub spans: Vec<Span>,
}

impl RenderedLine {
    pub fn text(&self) -> String {
        self.spans.iter().map(|span| span.text.as_str()).collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Rendered {
    pub lines: Vec<RenderedLine>,
}

impl Rendered {
    pub fn text(&self) -> String {
        self.lines
            .iter()
            .map(RenderedLine::text)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn to_source(&self, line: usize, col: usize) -> Option<usize> {
        let spans = &self.lines.get(line)?.spans;
        let mut offset = 0;
        for span in spans {
            let len = span.text.chars().count();
            if col < offset + len {
                if let Some((start, end)) = span.source {
                    let inside = col - offset;
                    if end - start == len {
                        return Some(start + inside);
                    }
                    return Some(start);
                }
                return None;
            }
            offset += len;
        }
        None
    }

    pub fn to_rendered(&self, source_idx: usize) -> (usize, usize) {
        let mut best = (0, 0);
        let mut best_start = None;
        for (line_no, line) in self.lines.iter().enumerate() {
            let mut offset = 0;
            for span in &line.spans {
                let len = span.text.chars().count();
                if let Some((start, end)) = span.source {
                    if source_idx >= start && source_idx < end && end - start == len {
                        return (line_no, offset + (source_idx - start));
                    }
                    if start <= source_idx && best_start.is_none_or(|b| start >= b) {
                        best_start = Some(start);
                        best = (
                            line_no,
                            offset + (source_idx - start).min(len.saturating_sub(1)),
                        );
                    }
                }
                offset += len;
            }
        }
        best
    }
}
