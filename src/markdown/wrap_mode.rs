use crate::markdown::blocks::BlockKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WrapMode {
    #[default]
    None,
    Text,
    All,
}

impl WrapMode {
    pub fn wraps(self, kind: &BlockKind) -> bool {
        match self {
            WrapMode::None => false,
            WrapMode::Text => matches!(
                kind,
                BlockKind::Paragraph { .. } | BlockKind::Quote { .. } | BlockKind::List { .. }
            ),
            WrapMode::All => !matches!(kind, BlockKind::Table { .. } | BlockKind::Verbatim { .. }),
        }
    }

    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "none" => Some(WrapMode::None),
            "text" => Some(WrapMode::Text),
            "all" => Some(WrapMode::All),
            _ => None,
        }
    }
}
