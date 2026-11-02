use super::argument::Arguments;
use super::{Access, Argument, Error};

#[derive(Debug)]
pub struct SetCommand {
    pub access: Access,
    parameters: String,
}

impl SetCommand {
    pub fn parse(input: &str) -> Option<Self> {
        let end = input.find([' ', '\t']).unwrap_or(input.len());
        let access = match &input[..end] {
            "set" => Access::Effective,
            "setlocal" => Access::Local,
            "setglobal" => Access::Global,
            _ => return None,
        };
        Some(Self {
            access,
            parameters: input[end..].to_string(),
        })
    }

    pub fn arguments(&self) -> impl Iterator<Item = Result<Argument, Error>> + '_ {
        Arguments::new(&self.parameters)
    }
}
