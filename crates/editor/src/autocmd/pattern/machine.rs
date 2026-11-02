use super::{Pattern, class::Class, unicode::fold};

#[derive(Debug, Clone)]
pub(super) enum Atom {
    Literal(char),
    Any,
    Class(Class),
}

#[derive(Debug, Clone, Default)]
pub(super) struct Node {
    pub epsilon: Vec<usize>,
    pub edges: Vec<(Atom, usize)>,
}

impl Atom {
    fn matches(&self, ch: char, ignore_case: bool) -> bool {
        match self {
            Self::Any => true,
            Self::Literal(literal) => *literal == ch || (ignore_case && fold(*literal) == fold(ch)),
            Self::Class(class) => class.matches(ch, ignore_case),
        }
    }
}

impl Pattern {
    pub(super) fn run(&self, name: &str, ignore_case: bool) -> bool {
        let mut seen = vec![false; self.nodes.len()];
        let mut active = Vec::new();
        let mut pending = vec![0];
        self.expand(&mut pending, &mut active, &mut seen);
        for ch in name.chars() {
            for &index in &active {
                for (atom, target) in &self.nodes[index].edges {
                    if atom.matches(ch, ignore_case) {
                        pending.push(*target);
                    }
                }
            }
            if pending.is_empty() {
                return false;
            }
            active.clear();
            seen.fill(false);
            self.expand(&mut pending, &mut active, &mut seen);
        }
        active.contains(&self.end)
    }

    fn expand(&self, pending: &mut Vec<usize>, active: &mut Vec<usize>, seen: &mut [bool]) {
        while let Some(index) = pending.pop() {
            if seen[index] {
                continue;
            }
            seen[index] = true;
            active.push(index);
            pending.extend(self.nodes[index].epsilon.iter().copied());
        }
    }
}
