use super::Error;
use std::cmp::Ordering;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EventName(&'static str);

impl EventName {
    pub fn all() -> impl Iterator<Item = Self> {
        NAMES.iter().copied().map(Self)
    }

    pub fn parse(name: &str) -> Result<Self, Error> {
        let canonical = ALIASES
            .iter()
            .find(|(alias, _)| alias.eq_ignore_ascii_case(name))
            .map_or(name, |(_, canonical)| *canonical);
        NAMES
            .iter()
            .find(|candidate| candidate.eq_ignore_ascii_case(canonical))
            .copied()
            .map(Self)
            .ok_or_else(|| Error::InvalidEvent(name.to_string()))
    }

    pub fn as_str(self) -> &'static str {
        self.0
    }
}

impl Ord for EventName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .bytes()
            .map(|byte| byte.to_ascii_lowercase())
            .cmp(other.0.bytes().map(|byte| byte.to_ascii_lowercase()))
    }
}

impl PartialOrd for EventName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

const ALIASES: &[(&str, &str)] = &[
    ("BufCreate", "BufAdd"),
    ("BufRead", "BufReadPost"),
    ("BufWrite", "BufWritePre"),
    ("FileEncoding", "EncodingChanged"),
];

const NAMES: &[&str] = &[
    "BufAdd",
    "BufDelete",
    "BufEnter",
    "BufFilePost",
    "BufFilePre",
    "BufHidden",
    "BufLeave",
    "BufModifiedSet",
    "BufNew",
    "BufNewFile",
    "BufReadCmd",
    "BufReadPost",
    "BufReadPre",
    "BufUnload",
    "BufWinEnter",
    "BufWinLeave",
    "BufWipeout",
    "BufWriteCmd",
    "BufWritePost",
    "BufWritePre",
    "ChanInfo",
    "ChanOpen",
    "CmdUndefined",
    "CmdlineChanged",
    "CmdlineEnter",
    "CmdlineLeave",
    "CmdlineLeavePre",
    "CmdwinEnter",
    "CmdwinLeave",
    "ColorScheme",
    "ColorSchemePre",
    "CompleteChanged",
    "CompleteDone",
    "CompleteDonePre",
    "CursorHold",
    "CursorHoldI",
    "CursorMoved",
    "CursorMovedC",
    "CursorMovedI",
    "DiagnosticChanged",
    "DiffUpdated",
    "DirChanged",
    "DirChangedPre",
    "EncodingChanged",
    "ExitPre",
    "FileAppendCmd",
    "FileAppendPost",
    "FileAppendPre",
    "FileChangedRO",
    "FileChangedShell",
    "FileChangedShellPost",
    "FileReadCmd",
    "FileReadPost",
    "FileReadPre",
    "FileType",
    "FileWriteCmd",
    "FileWritePost",
    "FileWritePre",
    "FilterReadPost",
    "FilterReadPre",
    "FilterWritePost",
    "FilterWritePre",
    "FocusGained",
    "FocusLost",
    "FuncUndefined",
    "GUIEnter",
    "GUIFailed",
    "InsertChange",
    "InsertCharPre",
    "InsertEnter",
    "InsertLeave",
    "InsertLeavePre",
    "LspAttach",
    "LspDetach",
    "LspNotify",
    "LspProgress",
    "LspRequest",
    "LspTokenUpdate",
    "MarkSet",
    "MenuPopup",
    "ModeChanged",
    "OptionSet",
    "PackChanged",
    "PackChangedPre",
    "Progress",
    "QuickFixCmdPost",
    "QuickFixCmdPre",
    "QuitPre",
    "RecordingEnter",
    "RecordingLeave",
    "RemoteReply",
    "SafeState",
    "SearchWrapped",
    "SessionLoadPost",
    "SessionLoadPre",
    "SessionWritePost",
    "ShellCmdPost",
    "ShellFilterPost",
    "Signal",
    "SourceCmd",
    "SourcePost",
    "SourcePre",
    "SpellFileMissing",
    "StdinReadPost",
    "StdinReadPre",
    "SwapExists",
    "Syntax",
    "TabClosed",
    "TabClosedPre",
    "TabEnter",
    "TabLeave",
    "TabNew",
    "TabNewEntered",
    "TermChanged",
    "TermClose",
    "TermEnter",
    "TermLeave",
    "TermOpen",
    "TermRequest",
    "TermResponse",
    "TextChanged",
    "TextChangedI",
    "TextChangedP",
    "TextChangedT",
    "TextYankPost",
    "UIEnter",
    "UILeave",
    "User",
    "VimEnter",
    "VimLeave",
    "VimLeavePre",
    "VimResized",
    "VimResume",
    "VimSuspend",
    "WinClosed",
    "WinEnter",
    "WinLeave",
    "WinNew",
    "WinNewPre",
    "WinResized",
    "WinScrolled",
];
