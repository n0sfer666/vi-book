#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trigger {
    Startup,
    Event(&'static str),
    Observe(&'static str),
    Filetype(&'static str),
}
