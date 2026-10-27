use v_ide_editor::disk::DiskMark;

pub fn suffix(mark: Option<DiskMark>) -> &'static str {
    match mark {
        Some(DiskMark::Changed) => " [changed]",
        Some(DiskMark::Deleted) => " [deleted]",
        None => "",
    }
}
