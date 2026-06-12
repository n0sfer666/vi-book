pub fn scroll_offset(selected: usize, height: usize) -> usize {
    (selected + 1).saturating_sub(height)
}

pub fn next(selected: usize, len: usize) -> usize {
    if len == 0 {
        0
    } else {
        (selected + 1).min(len - 1)
    }
}

pub fn prev(selected: usize) -> usize {
    selected.saturating_sub(1)
}
