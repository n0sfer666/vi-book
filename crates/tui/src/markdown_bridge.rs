use crate::app::App;
use crate::view::View;
use v_ide_markdown::module::MarkdownModule;
use v_ide_markdown::wrap_mode::WrapMode;

impl App {
    pub(crate) fn wrap(&self) -> WrapMode {
        self.modules
            .get::<MarkdownModule>()
            .map_or_else(|| MarkdownModule::default().wrap(), MarkdownModule::wrap)
    }

    #[cfg(feature = "lua")]
    pub(crate) fn set_wrap(&mut self, wrap: WrapMode) {
        let activated = self
            .modules
            .activate(&self.workspace, MarkdownModule::name());
        crate::requests::apply(self, activated);
        if let Some(markdown) = self.modules.get_mut::<MarkdownModule>() {
            markdown.set_wrap(wrap);
        }
    }

    pub(crate) fn readable(&self) -> bool {
        self.workspace.current().filetype() == Some("markdown")
    }

    pub(crate) fn initial_view(&self) -> View {
        if self.readable() {
            View::Read
        } else {
            View::Edit
        }
    }
}
