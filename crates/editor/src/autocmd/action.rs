use std::rc::Rc;

use crate::execution::{Context, Error};

use super::{FunctionRef, Invocation};

pub trait Callback {
    fn run(&self, cx: &Context<'_>, invocation: &Invocation<'_>) -> Result<bool, String>;

    fn function(&self) -> Option<FunctionRef> {
        None
    }
}

impl<F> Callback for F
where
    F: Fn(&Context<'_>, &Invocation<'_>) -> Result<bool, String>,
{
    fn run(&self, cx: &Context<'_>, invocation: &Invocation<'_>) -> Result<bool, String> {
        self(cx, invocation)
    }
}

#[derive(Clone)]
pub enum Action {
    Command(Rc<str>),
    Callback(Rc<dyn Callback>),
}

pub(super) enum Failure {
    Reported,
    Message(String),
}

impl Action {
    pub fn callback(callback: impl Callback + 'static) -> Self {
        Self::Callback(Rc::new(callback))
    }

    pub fn command(&self) -> Option<&str> {
        match self {
            Self::Command(command) => Some(command),
            Self::Callback(_) => None,
        }
    }

    pub fn function(&self) -> Option<FunctionRef> {
        match self {
            Self::Callback(callback) => callback.function(),
            Self::Command(_) => None,
        }
    }

    pub(super) fn run(
        &self,
        cx: &Context<'_>,
        invocation: &Invocation<'_>,
    ) -> Result<bool, Failure> {
        match self {
            Self::Command(command) => {
                cx.execute(command)
                    .map(|_| false)
                    .map_err(|error| match error {
                        Error::Autocmd(_) => Failure::Reported,
                        error => Failure::Message(error.to_string()),
                    })
            }
            Self::Callback(callback) => callback.run(cx, invocation).map_err(Failure::Message),
        }
    }
}
