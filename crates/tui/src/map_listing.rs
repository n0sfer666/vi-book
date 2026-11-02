use unicode_width::UnicodeWidthStr;
use v_ide_keys::{Key, MapMode, Rhs, format_keys};

use crate::{app::App, map_action::MapAction};

pub(crate) fn output(app: &mut App, modes: &[MapMode], prefix: &[Key]) {
    let lines: Vec<_> = app
        .keymap
        .list(modes, prefix)
        .iter()
        .map(|group| {
            let modes = mode_label(&group.modes);
            let lhs = format_keys(&group.lhs);
            let padding = " ".repeat(12usize.saturating_sub(lhs.width()).max(1));
            let marker = if group.mapping.remap { " " } else { "*" };
            format!(
                "{modes:2} {lhs}{padding}{marker} {}",
                rhs(&group.mapping.rhs)
            )
        })
        .collect();
    app.ui.set_message(if lines.is_empty() {
        String::from("No mapping found")
    } else {
        lines.join("\n")
    });
}

fn mode_label(modes: &[MapMode]) -> String {
    use MapMode::{Cmdline, Insert, Normal, OpPending, Visual};
    if modes.len() == 3
        && [Normal, Visual, OpPending]
            .iter()
            .all(|m| modes.contains(m))
    {
        return String::new();
    }
    if modes.len() == 2 && modes.contains(&Insert) && modes.contains(&Cmdline) {
        return String::from("!");
    }
    [
        (Normal, 'n'),
        (OpPending, 'o'),
        (Visual, 'x'),
        (Insert, 'i'),
        (Cmdline, 'c'),
    ]
    .iter()
    .filter_map(|(mode, label)| modes.contains(mode).then_some(*label))
    .collect()
}

fn rhs(value: &Rhs<MapAction>) -> String {
    let text = match value {
        Rhs::Keys(keys) => format_keys(keys),
        Rhs::Action(MapAction::Command(command)) => format!("<Cmd>{command}<CR>"),
        Rhs::Sequence(parts) => parts
            .iter()
            .map(rhs)
            .filter(|part| part != "<Nop>")
            .collect(),
    };
    if text.is_empty() {
        String::from("<Nop>")
    } else {
        text
    }
}
