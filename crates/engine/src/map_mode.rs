use crate::editor::Editor;
use crate::mode::Mode;
use crate::pending::Stage;
use v_ide_keys::MapMode;

impl Editor {
    pub fn map_mode(&self) -> Option<MapMode> {
        if self.search.input.is_some() {
            return Some(MapMode::Cmdline);
        }
        match (self.mode, self.pending.stage) {
            (Mode::Insert, _) => Some(MapMode::Insert),
            (_, Stage::Register | Stage::G | Stage::OperatorG(_) | Stage::OperatorObject(..)) => {
                None
            }
            (_, Stage::Operator(_)) => Some(MapMode::OpPending),
            (Mode::Normal, Stage::Start) => Some(MapMode::Normal),
            (Mode::Visual | Mode::VisualLine | Mode::VisualBlock, Stage::Start) => {
                Some(MapMode::Visual)
            }
        }
    }

    pub fn counting(&self) -> bool {
        match self.map_mode() {
            Some(MapMode::Normal | MapMode::Visual) => self.pending.count.is_some(),
            Some(MapMode::OpPending) => self.pending.op_count.is_some(),
            Some(MapMode::Insert | MapMode::Cmdline) | None => false,
        }
    }

    pub fn discard_count(&mut self) {
        match self.map_mode() {
            Some(MapMode::OpPending) => self.pending.op_count = None,
            _ => self.pending.count = None,
        }
    }
}
