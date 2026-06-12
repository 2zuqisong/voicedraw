pub mod canvas_state;
pub mod node_ops;
pub mod edge_ops;
pub mod layout;
pub mod style_ops;
pub mod snapshot;

pub use canvas_state::*;

use std::collections::HashMap;
use std::sync::Mutex;
use canvas_state::CanvasState;

/// 应用全局 Canvas 状态
pub struct AppEngine {
    pub canvas: Mutex<Option<CanvasState>>,
    pub snapshots: Mutex<snapshot::SnapshotManager>,
}

impl AppEngine {
    pub fn new() -> Self {
        // 创建默认画布
        let default_canvas = CanvasState {
            id: "default".into(),
            title: "未命名图表".into(),
            nodes: HashMap::new(),
            edges: HashMap::new(),
            theme: Theme::Default,
            width: 1200.0,
            height: 800.0,
        };
        Self {
            canvas: Mutex::new(Some(default_canvas)),
            snapshots: Mutex::new(snapshot::SnapshotManager::new(20)),
        }
    }
}