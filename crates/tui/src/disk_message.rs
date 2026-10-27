use v_ide_editor::disk::Reaction;

pub fn message(name: &str, reaction: Reaction) -> String {
    match reaction {
        Reaction::Reload => format!("\"{name}\" reloaded"),
        Reaction::Conflict => format!("\"{name}\" changed on disk (:e! load, :w! overwrite)"),
        Reaction::Vanished => format!("\"{name}\" deleted on disk"),
    }
}

pub fn missing(name: &str) -> String {
    format!("\"{name}\" not on disk")
}

pub fn summary(events: &[(String, Reaction)]) -> Option<String> {
    let (name, reaction) = events.iter().rev().max_by_key(|(_, r)| rank(*r))?;
    let text = message(name, *reaction);
    Some(match events.len() - 1 {
        0 => text,
        more => format!("{text} (+{more} more)"),
    })
}

fn rank(reaction: Reaction) -> u8 {
    match reaction {
        Reaction::Reload => 0,
        Reaction::Vanished => 1,
        Reaction::Conflict => 2,
    }
}
