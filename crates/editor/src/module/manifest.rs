use crate::module::Trigger;

#[derive(Debug)]
pub struct Manifest {
    pub name: &'static str,
    pub commands: &'static [&'static str],
    pub triggers: &'static [Trigger],
}
