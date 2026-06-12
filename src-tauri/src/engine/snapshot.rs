use super::canvas_state::CanvasState;

/// Undo/Redo 快照管理器
pub struct SnapshotManager {
    undo_stack: Vec<CanvasState>,
    redo_stack: Vec<CanvasState>,
    max_size: usize,
}

impl SnapshotManager {
    pub fn new(max_size: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_size,
        }
    }

    /// 保存当前状态到 undo 栈，清空 redo 栈
    pub fn save(&mut self, state: CanvasState) {
        self.undo_stack.push(state);
        self.redo_stack.clear();
        if self.undo_stack.len() > self.max_size {
            self.undo_stack.remove(0);
        }
    }

    /// 撤销：弹出 undo 栈顶，当前状态压入 redo
    pub fn undo(&mut self, current: CanvasState) -> Option<CanvasState> {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(current);
            Some(prev)
        } else {
            None
        }
    }

    /// 重做：弹出 redo 栈顶，当前状态压入 undo
    pub fn redo(&mut self, current: CanvasState) -> Option<CanvasState> {
        if let Some(next) = self.redo_stack.pop() {
            self.undo_stack.push(current);
            Some(next)
        } else {
            None
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::canvas_state::{CanvasState, Theme};
    use std::collections::HashMap;

    fn make_state(title: &str) -> CanvasState {
        CanvasState {
            id: "test".into(),
            title: title.into(),
            nodes: HashMap::new(),
            edges: HashMap::new(),
            theme: Theme::Default,
            width: 800.0,
            height: 600.0,
        }
    }

    #[test]
    fn test_undo_redo_cycle() {
        let mut mgr = SnapshotManager::new(10);
        let s1 = make_state("state1");
        let s2 = make_state("state2");

        mgr.save(s1);
        let result = mgr.undo(s2);
        assert!(result.is_some());
        assert_eq!(result.unwrap().title, "state1");
    }

    #[test]
    fn test_undo_empty_returns_none() {
        let mut mgr = SnapshotManager::new(10);
        let current = make_state("current");
        assert!(mgr.undo(current).is_none());
    }

    #[test]
    fn test_max_size() {
        let mut mgr = SnapshotManager::new(2);
        mgr.save(make_state("s1"));
        mgr.save(make_state("s2"));
        mgr.save(make_state("s3")); // should evict s1
        // undo twice should only get s3->s2, no s1
        let cur = make_state("cur");
        let r1 = mgr.undo(cur.clone());
        assert_eq!(r1.unwrap().title, "s3");
        let r2 = mgr.undo(cur);
        assert_eq!(r2.unwrap().title, "s2");
        let r3 = mgr.undo(make_state("after"));
        assert!(r3.is_none());
    }
}