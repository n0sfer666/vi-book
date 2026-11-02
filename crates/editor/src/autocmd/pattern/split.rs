pub fn patterns(source: &str) -> impl Iterator<Item = &str> {
    let mut remaining = source;
    std::iter::from_fn(move || {
        while !remaining.is_empty() {
            let mut depth = 0usize;
            let mut escaped = false;
            let mut end = remaining.len();
            for (index, ch) in remaining.char_indices() {
                if escaped {
                    escaped = false;
                    continue;
                }
                match ch {
                    '\\' => escaped = true,
                    '{' => depth = depth.saturating_add(1),
                    '}' => depth = depth.saturating_sub(1),
                    ',' if depth == 0 => {
                        end = index;
                        break;
                    }
                    _ => {}
                }
            }
            let item = &remaining[..end];
            remaining = remaining.get(end + 1..).unwrap_or_default();
            if !item.is_empty() {
                return Some(item);
            }
        }
        None
    })
}
