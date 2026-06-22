use crate::ui::app::App;

impl App {
    pub fn switch_to_buffer(&mut self, idx: usize) {
        if idx >= self.buffers.len() {
            return;
        }
        let old = self.take_active();
        self.buffers.insert(0, old);
        let target = self.buffers.remove(idx + 1);
        let path = target.path.clone();
        self.put_active(target);
        self.left = 0;
        self.history.visit(&path);
    }
}
