use std::any::Any;

mod cx;
mod entry;
mod event;
mod manifest;
mod modules;
mod registration;
mod request;
mod trigger;
mod vault_query;

pub use cx::Cx;
pub use event::Event;
pub use manifest::Manifest;
pub use modules::Modules;
pub use registration::Registration;
pub use request::Request;
pub use trigger::Trigger;
pub use vault_query::VaultQuery;

pub trait Module: Any {
    fn activate(&mut self, cx: &mut Cx<'_>) -> Result<(), String>;
    fn command(&mut self, cx: &mut Cx<'_>, name: &str, args: &str) -> Result<(), String>;
    fn event(&mut self, cx: &mut Cx<'_>, ev: &Event);

    fn tick(&mut self, _cx: &mut Cx<'_>) {}

    fn commands(&self) -> Vec<String> {
        Vec::new()
    }
}
