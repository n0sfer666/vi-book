use v_ide_editor::module::{Cx, Event, Manifest, Module, Registration, Trigger};

use crate::wrap_mode::WrapMode;

static MANIFEST: Manifest = Manifest {
    name: "markdown",
    commands: &["wrap"],
    triggers: &[Trigger::Filetype("markdown")],
};

pub struct MarkdownModule {
    wrap: WrapMode,
}

impl Default for MarkdownModule {
    fn default() -> Self {
        Self {
            wrap: WrapMode::All,
        }
    }
}

impl MarkdownModule {
    pub fn registration() -> Registration {
        Registration {
            manifest: &MANIFEST,
            make: || Box::new(Self::default()),
        }
    }

    pub fn name() -> &'static str {
        MANIFEST.name
    }

    pub fn wrap(&self) -> WrapMode {
        self.wrap
    }

    pub fn set_wrap(&mut self, wrap: WrapMode) {
        self.wrap = wrap;
    }
}

impl Module for MarkdownModule {
    fn activate(&mut self, _cx: &mut Cx<'_>) -> Result<(), String> {
        Ok(())
    }

    fn command(&mut self, cx: &mut Cx<'_>, _name: &str, args: &str) -> Result<(), String> {
        let mode = args.trim();
        let wrap = WrapMode::parse(mode)
            .ok_or_else(|| format!("unknown wrap mode: {mode} (none|text|all)"))?;
        self.wrap = wrap;
        cx.message(format!("wrap: {mode}"));
        Ok(())
    }

    fn event(&mut self, _cx: &mut Cx<'_>, _ev: &Event) {}
}
