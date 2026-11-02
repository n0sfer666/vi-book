use super::PluginHost;
use crate::config::Keymap;
use mlua::Table;

impl PluginHost {
    pub fn keymaps(&self) -> Vec<Keymap> {
        let Ok(maps) = self.vibook_field("__keymaps") else {
            return Vec::new();
        };
        maps.sequence_values::<Table>()
            .filter_map(Result::ok)
            .filter_map(|t| {
                Some(Keymap {
                    mode: match t.get::<mlua::Value>("mode").ok()? {
                        mlua::Value::String(mode) => vec![mode.to_str().ok()?.to_string()],
                        mlua::Value::Table(modes) => {
                            modes.sequence_values().collect::<Result<_, _>>().ok()?
                        }
                        _ => return None,
                    },
                    lhs: t.get("lhs").ok()?,
                    rhs: t.get("rhs").ok()?,
                    remap: t.get("remap").unwrap_or(false),
                    nowait: t.get("nowait").unwrap_or(false),
                    unique: t.get("unique").unwrap_or(false),
                })
            })
            .collect()
    }

    pub fn keymap_errors(&self) -> Vec<String> {
        self.vibook_field("__keymap_errors")
            .map(|list| list.sequence_values().filter_map(Result::ok).collect())
            .unwrap_or_default()
    }
}
