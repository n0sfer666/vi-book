#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WikiLink {
    pub name: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    pub name: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frontmatter {
    pub text: String,
    pub end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NoteFacts {
    pub links: Vec<WikiLink>,
    pub tags: Vec<Tag>,
    pub frontmatter: Option<Frontmatter>,
}
