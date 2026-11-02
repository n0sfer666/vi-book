pub(super) struct Scan {
    ends: Vec<Option<usize>>,
    named: Vec<Option<usize>>,
}

impl Scan {
    pub(super) fn new(chars: &[char]) -> Self {
        let mut named = vec![None; chars.len() + 1];
        let mut ends = named.clone();
        for index in (0..chars.len()).rev() {
            named[index] = if chars.get(index..index + 2) == Some(&[':', ']']) {
                Some(index + 2)
            } else {
                named[index + 1]
            };
            ends[index] = match chars[index] {
                '[' if chars.get(index + 1) == Some(&':') => {
                    named[index + 2].map_or(ends[index + 1], |after| ends[after])
                }
                '\\' if chars.get(index + 1).is_some_and(|ch| "\\]-^".contains(*ch)) => {
                    ends[index + 2]
                }
                '-' if chars.get(index + 1).is_some_and(|ch| *ch != ']') => ends[index + 2],
                ']' => Some(index),
                _ => ends[index + 1],
            };
        }
        Self { ends, named }
    }

    pub(super) fn class_end(&self, start: usize) -> Option<usize> {
        self.ends.get(start).copied().flatten()
    }

    pub(super) fn named_end(&self, start: usize, limit: usize) -> Option<usize> {
        self.named
            .get(start)
            .copied()
            .flatten()
            .filter(|&end| end <= limit)
    }
}
