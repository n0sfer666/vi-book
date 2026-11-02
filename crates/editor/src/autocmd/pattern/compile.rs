use super::{
    Pattern, PatternError,
    class::Class,
    machine::{Atom, Node},
    scan::Scan,
};

struct Group {
    entry: usize,
    exits: Vec<usize>,
}

pub(super) fn compile(source: &str) -> Result<Pattern, PatternError> {
    let chars: Vec<_> = source.chars().collect();
    let scan = Scan::new(&chars);
    let mut nodes = vec![Node::default()];
    let mut groups: Vec<Group> = Vec::new();
    let mut current = 0;
    let mut index = 0;
    while let Some(&ch) = chars.get(index) {
        index += 1;
        match ch {
            '*' => nodes[current].edges.push((Atom::Any, current)),
            '{' => {
                let branch = node(&mut nodes);
                nodes[current].epsilon.push(branch);
                groups.push(Group {
                    entry: current,
                    exits: Vec::new(),
                });
                current = branch;
            }
            ',' if !groups.is_empty() => {
                let group = groups.last_mut().unwrap();
                group.exits.push(current);
                current = node(&mut nodes);
                nodes[group.entry].epsilon.push(current);
            }
            '}' => {
                let mut group = groups.pop().ok_or(PatternError::MissingOpen)?;
                group.exits.push(current);
                current = node(&mut nodes);
                for exit in group.exits {
                    nodes[exit].epsilon.push(current);
                }
            }
            _ => {
                let atom = match ch {
                    '?' => Atom::Any,
                    '[' => Class::parse(&chars, &mut index, &scan)?
                        .map_or(Atom::Literal('['), Atom::Class),
                    '\\' => {
                        let Some(&escaped) = chars.get(index) else {
                            break;
                        };
                        if is_regex(escaped)
                            || chars.get(index..index + 3) == Some(&['\\', '\\', '{'])
                        {
                            return Err(PatternError::UnsupportedRegex(format!("\\{escaped}")));
                        }
                        index += 1;
                        Atom::Literal(escaped)
                    }
                    literal => Atom::Literal(literal),
                };
                let next = node(&mut nodes);
                nodes[current].edges.push((atom, next));
                current = next;
            }
        }
    }
    if !groups.is_empty() {
        return Err(PatternError::MissingClose);
    }
    Ok(Pattern {
        nodes,
        end: current,
        directories: source.contains('/'),
    })
}

fn node(nodes: &mut Vec<Node>) -> usize {
    let id = nodes.len();
    nodes.push(Node::default());
    id
}

fn is_regex(ch: char) -> bool {
    ch.is_ascii_digit() || "aAbcCdDefFhHiIkKlLmMnopPrsStuUvVwWxXzZ()|+=@_&<>".contains(ch)
}
