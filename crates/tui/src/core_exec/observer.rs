use crate::app::App;
use std::rc::Rc;
use v_ide_editor::{
    autocmd::Bus,
    execution::{Context, Error, Observer as Observe, OptionSet},
};

pub(super) struct Observer {
    bus: Rc<Bus>,
    #[cfg(feature = "lua")]
    host: Option<Rc<v_ide_lua::host::PluginHost>>,
}
impl Observer {
    pub fn new(_app: &App, bus: Rc<Bus>) -> Self {
        Self {
            bus,
            #[cfg(feature = "lua")]
            host: _app
                .modules
                .get::<v_ide_lua::module::LuaModule>()
                .and_then(v_ide_lua::module::LuaModule::owned_host),
        }
    }
}
impl Observe for Observer {
    fn autocmd_bus(&self) -> Option<&Bus> {
        Some(self.bus.as_ref())
    }
    fn option_set(&self, cx: &Context<'_>, event: &OptionSet) -> Result<(), String> {
        self.bus.option_set(cx, event)
    }
    fn command(&self, _cx: &Context<'_>, _input: &str) -> Option<Result<(), Error>> {
        #[cfg(feature = "lua")]
        if let Some(host) = &self.host {
            let name = _input.split_ascii_whitespace().next()?;
            let checkpoint = _cx.diagnostic_checkpoint();
            return match host.call_with_context(_cx, |host| host.run_command(name)) {
                Ok(true) => Some(Ok(())),
                Ok(false) => None,
                Err(error) => Some(Err(if error.is_reported_since(checkpoint) {
                    Error::Autocmd(error.to_string())
                } else {
                    Error::Ex(error.to_string())
                })),
            };
        }
        None
    }
}
