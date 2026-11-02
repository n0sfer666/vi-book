use super::EventName;

impl EventName {
    pub fn is_file_pattern(self) -> bool {
        !matches!(
            self.as_str(),
            "CmdUndefined"
                | "CmdlineChanged"
                | "CmdlineEnter"
                | "CmdlineLeave"
                | "CmdlineLeavePre"
                | "CmdwinEnter"
                | "CmdwinLeave"
                | "ColorScheme"
                | "ColorSchemePre"
                | "CursorMovedC"
                | "DirChanged"
                | "DirChangedPre"
                | "FileType"
                | "FuncUndefined"
                | "MarkSet"
                | "MenuPopup"
                | "ModeChanged"
                | "OptionSet"
                | "Progress"
                | "QuickFixCmdPost"
                | "QuickFixCmdPre"
                | "RemoteReply"
                | "Signal"
                | "SpellFileMissing"
                | "Syntax"
                | "TabClosed"
                | "User"
                | "WinClosed"
                | "WinResized"
                | "WinScrolled"
        )
    }
}
