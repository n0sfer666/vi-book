use crate::ui::app::App;
use crate::vault::follow::{Follow, follow, link_at};

pub fn follow_at_cursor(app: &mut App) {
    let Some(link) = link_at(&app.editor.text(), app.editor.text_index()) else {
        return;
    };
    match follow(&app.session.index, &link) {
        Follow::Open(path) => app.open_rel(&path),
        Follow::Disambiguate(paths) => {
            app.picker = Some(crate::ui::picker::Picker::new(paths));
        }
        Follow::Create(name) => {
            app.message = format!("note '{name}' does not exist (press : to create)");
        }
    }
}
