use super::Request;
use crate::execution::Context;
use std::{fmt, rc::Rc};

type Run = dyn Fn(&Context<'_>) -> Vec<Request>;

#[derive(Clone)]
pub struct Action(Rc<Run>);

impl Action {
    pub fn new(run: impl Fn(&Context<'_>) -> Vec<Request> + 'static) -> Self {
        Self(Rc::new(run))
    }
    pub fn run(&self, cx: &Context<'_>) -> Vec<Request> {
        (self.0)(cx)
    }
}
impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Action")
    }
}
impl PartialEq for Action {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for Action {}
