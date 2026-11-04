use std::collections::HashSet;

use super::{Buffer, DocId, WindowId, Workspace};

pub struct EventScope {
    target: DocId,
    previous: DocId,
    window: WindowId,
    pool: Vec<Buffer>,
}

impl Workspace {
    pub fn enter_event_scope(&mut self, target: DocId) -> Option<EventScope> {
        self.documents.contains_key(&target).then(|| {
            let scope = EventScope {
                target,
                previous: self.current_id(),
                window: self.active_window(),
                pool: self.pool.clone(),
            };
            self.select_service(target);
            scope
        })
    }

    pub fn finish_event_scope(
        &mut self,
        scope: EventScope,
        explicit_navigation: bool,
    ) -> Option<DocId> {
        if explicit_navigation
            || self.active_window() != scope.window
            || self.current_id() != scope.target
        {
            return None;
        }
        self.select_service(scope.previous);
        let mut pool = scope
            .pool
            .into_iter()
            .filter(|buffer| self.documents.contains_key(&buffer.id) && !self.visible(buffer.id))
            .collect::<Vec<_>>();
        let mut included = pool.iter().map(|buffer| buffer.id).collect::<HashSet<_>>();
        pool.extend(self.pool.iter().copied().filter(|buffer| {
            self.documents.contains_key(&buffer.id)
                && !self.visible(buffer.id)
                && included.insert(buffer.id)
        }));
        self.pool = pool;
        Some(scope.target)
    }

    pub fn select_service(&mut self, target: DocId) -> bool {
        if target == self.current_id() || !self.documents.contains_key(&target) {
            return false;
        }
        let old = self.activate(Buffer { id: target });
        self.park(old);
        true
    }
}
