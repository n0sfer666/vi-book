use crate::module::{Cx, Module, Registration, Trigger};

pub struct Entry {
    registration: Registration,
    state: Option<Box<dyn Module>>,
}

impl Entry {
    pub fn new(registration: Registration) -> Self {
        Self {
            registration,
            state: None,
        }
    }

    pub fn handles(&self, command: &str) -> bool {
        self.registration.manifest.commands.contains(&command)
    }

    pub fn listens(&self, event: &str) -> bool {
        self.triggers()
            .iter()
            .any(|t| matches!(t, Trigger::Event(name) if *name == event))
    }

    pub fn on_startup(&self) -> bool {
        self.triggers().contains(&Trigger::Startup)
    }

    pub fn active(&mut self, cx: &mut Cx<'_>) -> Option<&mut Box<dyn Module>> {
        if self.state.is_none() {
            let mut module = (self.registration.make)();
            if let Err(err) = module.activate(cx) {
                cx.message(err);
                return None;
            }
            self.state = Some(module);
        }
        self.state.as_mut()
    }

    pub fn module(&self) -> Option<&dyn Module> {
        self.state.as_deref()
    }

    pub fn module_mut(&mut self) -> Option<&mut dyn Module> {
        self.state.as_deref_mut()
    }

    fn triggers(&self) -> &'static [Trigger] {
        self.registration.manifest.triggers
    }
}
