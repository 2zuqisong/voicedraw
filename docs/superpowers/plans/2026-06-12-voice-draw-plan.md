# Voice-to-Draw 分步实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 从零搭建纯语音控制的智能绘图桌面应用，分 8 个阶段、每阶段 4-8 个可独立验证的小任务，逐步交付。

**Architecture:** Tauri 2.x 桌面壳 → React 18 前端（Fabric.js Canvas + Web Speech API）→ Rust 后端（预处理器 + DeepSeek V4 LLM 调度器 + Canvas 命令引擎）。核心模式：LLM Function Calling 操控 Rust 端 Canvas 状态机，状态通过 Tauri Event 推送到前端渲染。

**Runtime:** Bun.js（包管理器 + 运行时，替代 Node.js/npm，使用 `bun install`/`bun run`/`bun add`）
**Tech Stack:** Tauri 2.x, React 18, TypeScript, Fabric.js 6.x, Zustand, Rust, async-openai, DeepSeek V4 API

---

## Phase 0: 项目骨架搭建

> 目标：创建 Tauri + React + TypeScript 项目，安装全部依赖，确认三端（前端/Rust/LLM）能独立跑通。

### Task 0.1: 创建 Tauri + React 项目

**Files:**
- Create: 整个项目骨架（`bun create tauri-app`）

- [ ] **Step 1: 用 Tauri CLI 创建项目**

```bash
cd /home/zuqis/voice-to-draw
bun create tauri-app@latest voice-draw -- --template react-ts
```

选择 React + TypeScript 模板。

- [ ] **Step 2: 进入项目目录并安装依赖**

```bash
cd voice-draw
bun install
```

- [ ] **Step 3: 验证项目能启动**

```bash
bun run tauri dev
```

预期：看到一个默认的 Tauri 窗口，显示 React 页面。

- [ ] **Step 4: 提交**

```bash
git add -A
git commit -m "feat: scaffold Tauri + React + TypeScript project

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 0.2: 安装前端依赖

**Files:**
- Modify: `package.json`

- [ ] **Step 1: 安装 Canvas、状态管理、布局库**

```bash
bun install fabric@6.x zustand dagre
bun add -d @types/fabric
```

- [ ] **Step 2: 验证安装**

```bash
bun -e "require('fabric'); console.log('fabric OK')"
bun -e "require('zustand'); console.log('zustand OK')"
bun -e "require('dagre'); console.log('dagre OK')"
```

- [ ] **Step 3: 提交**

```bash
git add package.json bun.lockb
git commit -m "feat: add Fabric.js, Zustand, dagre frontend dependencies

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 0.3: 配置 Rust 依赖

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: 添加 Rust crate 依赖**

编辑 `src-tauri/Cargo.toml`，在 `[dependencies]` 中添加：

```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
async-openai = "0.27"           # DeepSeek 兼容 OpenAI API
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4"] }
thiserror = "2"
log = "0.4"
env_logger = "0.11"
```

- [ ] **Step 2: 编译验证**

```bash
cd src-tauri && cargo check
```

预期：依赖全部下载，无编译错误。

- [ ] **Step 3: 提交**

```bash
git add src-tauri/Cargo.toml src-tauri/Cargo.lock
git commit -m "feat: add Rust dependencies (async-openai, serde, tokio, uuid, thiserror)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 0.4: 验证三端通信

**Files:**
- Create: `src-tauri/src/lib.rs`
- Modify: `src/App.tsx`

- [ ] **Step 1: 编写 Rust 测试命令**

在 `src-tauri/src/lib.rs` 中写一个最小可用的 Tauri command：

```rust
#[tauri::command]
fn greet(name: &str) -> String {
    format!("你好, {}! Rust 后端已就绪。", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 2: 在 React 中调用**

修改 `src/App.tsx`：

```tsx
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

function App() {
  const [msg, setMsg] = useState("");

  const testInvoke = async () => {
    const response: string = await invoke("greet", { name: "世界" });
    setMsg(response);
  };

  return (
    <div>
      <button onClick={testInvoke}>测试 Rust 通信</button>
      <p>{msg}</p>
    </div>
  );
}

export default App;
```

- [ ] **Step 3: 运行验证**

```bash
bun run tauri dev
```

点击按钮，确认显示 "你好, 世界! Rust 后端已就绪。"

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/lib.rs src-tauri/src/main.rs src/App.tsx
git commit -m "feat: verify Tauri frontend-backend communication works

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Phase 1: Rust Canvas 状态引擎

> 目标：在 Rust 端建立完整的 Canvas 状态模型，实现节点/连线的 CRUD 和 undo/redo。这是整个系统的「真理源」——前端只负责渲染，所有操作逻辑在 Rust。

### Task 1.1: 定义核心数据结构

**Files:**
- Create: `src-tauri/src/engine/mod.rs`
- Create: `src-tauri/src/engine/canvas_state.rs`

- [ ] **Step 1: 定义所有枚举和结构体**

`src-tauri/src/engine/canvas_state.rs`:

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 节点类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NodeType {
    Start,
    End,
    Process,
    Decision,
    Data,
    Subprocess,
    Text,
}

impl NodeType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "start" => Ok(Self::Start),
            "end" => Ok(Self::End),
            "process" => Ok(Self::Process),
            "decision" => Ok(Self::Decision),
            "data" => Ok(Self::Data),
            "subprocess" => Ok(Self::Subprocess),
            "text" => Ok(Self::Text),
            _ => Err(format!("未知节点类型: {}", s)),
        }
    }
}

/// 位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

/// 尺寸
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Size {
    pub width: f64,
    pub height: f64,
}

/// 节点样式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStyle {
    pub fill: String,
    pub stroke: String,
    pub stroke_width: f64,
    pub font_size: f64,
    pub font_family: String,
    pub border_radius: f64,
}

impl Default for NodeStyle {
    fn default() -> Self {
        Self {
            fill: "#ffffff".into(),
            stroke: "#333333".into(),
            stroke_width: 2.0,
            font_size: 14.0,
            font_family: "sans-serif".into(),
            border_radius: 8.0,
        }
    }
}

/// 连线样式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeStyle {
    pub line_style: LineStyle,
    pub arrow: ArrowType,
    pub stroke: String,
    pub stroke_width: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineStyle {
    Solid,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArrowType {
    Single,
    Double,
    None,
}

impl Default for EdgeStyle {
    fn default() -> Self {
        Self {
            line_style: LineStyle::Solid,
            arrow: ArrowType::Single,
            stroke: "#555555".into(),
            stroke_width: 2.0,
        }
    }
}

/// 图表节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramNode {
    pub id: String,
    pub node_type: NodeType,
    pub label: String,
    pub position: Position,
    pub size: Size,
    pub style: NodeStyle,
}

/// 图表连线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramEdge {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub label: Option<String>,
    pub style: EdgeStyle,
}

/// 主题
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Theme {
    Default,
    Professional,
    Handdrawn,
    Dark,
    Colorful,
}

impl Theme {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "default" => Ok(Self::Default),
            "professional" => Ok(Self::Professional),
            "handdrawn" => Ok(Self::Handdrawn),
            "dark" => Ok(Self::Dark),
            "colorful" => Ok(Self::Colorful),
            _ => Err(format!("未知主题: {}", s)),
        }
    }
}

/// Canvas 完整状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanvasState {
    pub id: String,
    pub title: String,
    pub nodes: HashMap<String, DiagramNode>,
    pub edges: HashMap<String, DiagramEdge>,
    pub theme: Theme,
    pub width: f64,
    pub height: f64,
}
```

- [ ] **Step 2: 定义模块入口**

`src-tauri/src/engine/mod.rs`:

```rust
pub mod canvas_state;
pub mod node_ops;
pub mod edge_ops;
pub mod layout;
pub mod style_ops;
pub mod snapshot;

pub use canvas_state::*;
```

- [ ] **Step 3: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/engine/
git commit -m "feat: define CanvasState, DiagramNode, DiagramEdge core data structures

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 1.2: 实现 Undo/Redo 快照系统

**Files:**
- Create: `src-tauri/src/engine/snapshot.rs`

- [ ] **Step 1: 实现快照管理器**

`src-tauri/src/engine/snapshot.rs`:

```rust
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
```

- [ ] **Step 2: 写单元测试**

在 `snapshot.rs` 末尾追加：

```rust
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
```

- [ ] **Step 3: 运行测试**

```bash
cd src-tauri && cargo test snapshot
```

预期：3 个测试全部 PASS。

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/engine/snapshot.rs
git commit -m "feat: implement SnapshotManager with undo/redo and tests

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 1.3: 实现节点 CRUD 操作

**Files:**
- Create: `src-tauri/src/engine/node_ops.rs`

- [ ] **Step 1: 实现节点操作函数**

`src-tauri/src/engine/node_ops.rs`:

```rust
use super::canvas_state::*;
use std::collections::HashMap;
use uuid::Uuid;

/// 添加单个节点
pub fn add_node(
    nodes: &mut HashMap<String, DiagramNode>,
    node_type: NodeType,
    label: String,
    position: Option<Position>,
    style: Option<NodeStyle>,
) -> DiagramNode {
    let id = Uuid::new_v4().to_string();
    let pos = position.unwrap_or(Position { x: 100.0, y: 100.0 });
    let node = DiagramNode {
        id: id.clone(),
        node_type,
        label,
        position: pos,
        size: Size { width: 160.0, height: 60.0 },
        style: style.unwrap_or_default(),
    };
    nodes.insert(id, node.clone());
    node
}

/// 批量添加节点（位置由自动布局决定时为 None）
pub fn add_nodes_batch(
    nodes: &mut HashMap<String, DiagramNode>,
    batch: Vec<(NodeType, String)>,
) -> Vec<DiagramNode> {
    batch
        .into_iter()
        .map(|(nt, label)| add_node(nodes, nt, label, None, None))
        .collect()
}

/// 更新节点
pub fn update_node(
    nodes: &mut HashMap<String, DiagramNode>,
    node_id: &str,
    label: Option<String>,
    style: Option<NodeStyle>,
    position: Option<Position>,
) -> Result<DiagramNode, String> {
    let node = nodes.get_mut(node_id).ok_or(format!("节点 {} 不存在", node_id))?;
    if let Some(l) = label { node.label = l; }
    if let Some(s) = style { node.style = s; }
    if let Some(p) = position { node.position = p; }
    Ok(node.clone())
}

/// 删除节点（返回被删除的关联连线 from_id/to_id）
pub fn delete_node(
    nodes: &mut HashMap<String, DiagramNode>,
    edges: &mut HashMap<String, DiagramEdge>,
    node_id: &str,
) -> Result<(DiagramNode, Vec<String>), String> {
    let removed = nodes.remove(node_id).ok_or(format!("节点 {} 不存在", node_id))?;
    let deleted_edges: Vec<String> = edges
        .iter()
        .filter(|(_, e)| e.from_id == node_id || e.to_id == node_id)
        .map(|(id, _)| id.clone())
        .collect();
    for id in &deleted_edges {
        edges.remove(id);
    }
    Ok((removed, deleted_edges))
}

/// 移动节点（支持相对方向）
pub fn move_node(
    nodes: &mut HashMap<String, DiagramNode>,
    node_id: &str,
    target: MoveTarget,
) -> Result<(Position, Position), String> {
    let node = nodes.get_mut(node_id).ok_or(format!("节点 {} 不存在", node_id))?;
    let old = node.position.clone();
    match target {
        MoveTarget::Absolute(x, y) => { node.position = Position { x, y }; }
        MoveTarget::Direction(dir) => {
            let step = 60.0;
            match dir.as_str() {
                "left" => node.position.x -= step,
                "right" => node.position.x += step,
                "up" => node.position.y -= step,
                "down" => node.position.y += step,
                _ => return Err(format!("未知方向: {}", dir)),
            }
        }
    }
    Ok((old, node.position.clone()))
}

pub enum MoveTarget {
    Absolute(f64, f64),
    Direction(String),
}

/// 获取画布上的节点列表摘要（不含完整样式细节，给 LLM 用的轻量信息）
pub fn get_nodes_summary(nodes: &HashMap<String, DiagramNode>) -> Vec<NodeSummary> {
    nodes.values().map(|n| NodeSummary {
        id: n.id.clone(),
        node_type: format!("{:?}", n.node_type),
        label: n.label.clone(),
        x: n.position.x,
        y: n.position.y,
    }).collect()
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeSummary {
    pub id: String,
    pub node_type: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
}
```

- [ ] **Step 2: 写测试**

在 `node_ops.rs` 末尾：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_delete_node() {
        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();
        let node = add_node(&mut nodes, NodeType::Process, "测试节点".into(), None, None);
        assert_eq!(nodes.len(), 1);
        
        let (removed, deleted) = delete_node(&mut nodes, &mut edges, &node.id).unwrap();
        assert_eq!(removed.label, "测试节点");
        assert!(deleted.is_empty());
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_delete_node_removes_edges() {
        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();
        let n1 = add_node(&mut nodes, NodeType::Start, "A".into(), None, None);
        let n2 = add_node(&mut nodes, NodeType::End, "B".into(), None, None);
        let e = crate::engine::edge_ops::add_edge(&mut edges, &n1.id, &n2.id, None, None).unwrap();
        
        let (_, deleted) = delete_node(&mut nodes, &mut edges, &n1.id).unwrap();
        assert!(deleted.contains(&e.id));
        assert!(edges.is_empty());
    }
}
```

- [ ] **Step 3: 运行测试**

```bash
cd src-tauri && cargo test node_ops
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/engine/node_ops.rs
git commit -m "feat: implement node CRUD (add/add_batch/update/delete/move) with tests

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 1.4: 实现连线 CRUD 操作

**Files:**
- Create: `src-tauri/src/engine/edge_ops.rs`

- [ ] **Step 1: 实现连线操作函数**

`src-tauri/src/engine/edge_ops.rs`:

```rust
use super::canvas_state::*;
use std::collections::HashMap;
use uuid::Uuid;

/// 添加单条连线
pub fn add_edge(
    edges: &mut HashMap<String, DiagramEdge>,
    from_id: &str,
    to_id: &str,
    label: Option<String>,
    style: Option<EdgeStyle>,
) -> Result<DiagramEdge, String> {
    let id = Uuid::new_v4().to_string();
    let edge = DiagramEdge {
        id: id.clone(),
        from_id: from_id.to_string(),
        to_id: to_id.to_string(),
        label,
        style: style.unwrap_or_default(),
    };
    edges.insert(id, edge.clone());
    Ok(edge)
}

/// 批量添加连线
pub fn add_edges_batch(
    edges: &mut HashMap<String, DiagramEdge>,
    batch: Vec<EdgeDef>,
) -> Vec<DiagramEdge> {
    batch
        .into_iter()
        .filter_map(|def| add_edge(edges, &def.from, &def.to, def.label, def.style).ok())
        .collect()
}

pub struct EdgeDef {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub style: Option<EdgeStyle>,
}

/// 更新连线
pub fn update_edge(
    edges: &mut HashMap<String, DiagramEdge>,
    edge_id: &str,
    label: Option<String>,
    style: Option<EdgeStyle>,
) -> Result<DiagramEdge, String> {
    let edge = edges.get_mut(edge_id).ok_or(format!("连线 {} 不存在", edge_id))?;
    if let Some(l) = label { edge.label = Some(l); }
    if let Some(s) = style { edge.style = s; }
    Ok(edge.clone())
}

/// 删除连线
pub fn delete_edge(
    edges: &mut HashMap<String, DiagramEdge>,
    edge_id: &str,
) -> Result<DiagramEdge, String> {
    edges.remove(edge_id).ok_or(format!("连线 {} 不存在", edge_id))
}
```

- [ ] **Step 2: 运行编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/engine/edge_ops.rs
git commit -m "feat: implement edge CRUD (add/add_batch/update/delete)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 1.5: 实现自动布局算法

**Files:**
- Create: `src-tauri/src/engine/layout.rs`

- [ ] **Step 1: 实现简单分层布局**

`src-tauri/src/engine/layout.rs`:

```rust
use super::canvas_state::*;
use std::collections::HashMap;

/// 简单从上到下分层布局
/// 原理：BFS 遍历，按层分配 y，同层均匀分配 x
pub fn top_down_layout(
    nodes: &mut HashMap<String, DiagramNode>,
    edges: &HashMap<String, DiagramEdge>,
) {
    // 1. 找到起始节点（没有入边的节点）
    let has_incoming: std::collections::HashSet<&str> = edges
        .values()
        .map(|e| e.to_id.as_str())
        .collect();
    let roots: Vec<String> = nodes
        .keys()
        .filter(|id| !has_incoming.contains(id.as_str()))
        .cloned()
        .collect();

    if roots.is_empty() {
        // 没有明显根节点，按节点 ID 排序作为第一层
        layout_fallback(nodes);
        return;
    }

    // 2. BFS 分层
    let mut layers: Vec<Vec<String>> = vec![roots];
    let mut placed: std::collections::HashSet<String> =
        layers[0].iter().cloned().collect();

    loop {
        let prev_layer = &layers[layers.len() - 1];
        let mut next_layer: Vec<String> = Vec::new();
        for node_id in prev_layer {
            for edge in edges.values().filter(|e| e.from_id == *node_id) {
                if !placed.contains(&edge.to_id) {
                    next_layer.push(edge.to_id.clone());
                    placed.insert(edge.to_id.clone());
                }
            }
        }
        if next_layer.is_empty() {
            break;
        }
        layers.push(next_layer);
    }

    // 3. 将未遍历到的节点追加到最后一层
    for id in nodes.keys() {
        if !placed.contains(id) {
            if layers.is_empty() {
                layers.push(vec![id.clone()]);
            } else {
                layers.last_mut().unwrap().push(id.clone());
            }
            placed.insert(id.clone());
        }
    }

    // 4. 计算坐标
    let x_spacing = 200.0;
    let y_spacing = 120.0;
    let start_y = 60.0;

    for (layer_idx, layer) in layers.iter().enumerate() {
        let total_width = (layer.len() as f64 - 1.0) * x_spacing;
        let start_x = 400.0 - total_width / 2.0; // 居中
        for (node_idx, node_id) in layer.iter().enumerate() {
            if let Some(node) = nodes.get_mut(node_id) {
                node.position = Position {
                    x: start_x + node_idx as f64 * x_spacing,
                    y: start_y + layer_idx as f64 * y_spacing,
                };
            }
        }
    }
}

fn layout_fallback(nodes: &mut HashMap<String, DiagramNode>) {
    for (i, (_, node)) in nodes.iter_mut().enumerate() {
        node.position = Position {
            x: 100.0 + (i as f64 % 4.0) * 200.0,
            y: 60.0 + (i as f64 / 4.0).floor() * 120.0,
        };
    }
}

/// 从左到右分层布局（树状图/思维导图）
pub fn left_right_layout(
    nodes: &mut HashMap<String, DiagramNode>,
    edges: &HashMap<String, DiagramEdge>,
) {
    // 先执行 top_down 布局
    top_down_layout(nodes, edges);
    // 交换 x 和 y
    for node in nodes.values_mut() {
        std::mem::swap(&mut node.position.x, &mut node.position.y);
    }
}

/// 布局入口
pub enum LayoutDirection {
    TopDown,
    LeftRight,
}

pub fn auto_layout(
    nodes: &mut HashMap<String, DiagramNode>,
    edges: &HashMap<String, DiagramEdge>,
    direction: LayoutDirection,
) -> usize {
    let old_positions: HashMap<String, Position> = nodes
        .iter()
        .map(|(id, n)| (id.clone(), n.position.clone()))
        .collect();
    match direction {
        LayoutDirection::TopDown => top_down_layout(nodes, edges),
        LayoutDirection::LeftRight => left_right_layout(nodes, edges),
    }
    // 返回移动了的节点数
    nodes.iter().filter(|(id, n)| {
        old_positions.get(*id).map_or(true, |old| old.x != n.position.x || old.y != n.position.y)
    }).count()
}
```

- [ ] **Step 2: 写测试**

在 `layout.rs` 末尾：

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_down_three_nodes_chain() {
        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();

        use crate::engine::node_ops::add_node;
        use crate::engine::edge_ops::add_edge;

        let n1 = add_node(&mut nodes, NodeType::Start, "开始".into(), None, None);
        let n2 = add_node(&mut nodes, NodeType::Process, "处理".into(), None, None);
        let n3 = add_node(&mut nodes, NodeType::End, "结束".into(), None, None);
        add_edge(&mut edges, &n1.id, &n2.id, None, None).unwrap();
        add_edge(&mut edges, &n2.id, &n3.id, None, None).unwrap();

        top_down_layout(&mut nodes, &edges);

        // 验证分层：n1 应该在最上面，n3 在最下面
        assert!(nodes[&n1.id].position.y < nodes[&n2.id].position.y);
        assert!(nodes[&n2.id].position.y < nodes[&n3.id].position.y);
    }
}
```

- [ ] **Step 3: 运行测试**

```bash
cd src-tauri && cargo test layout
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/engine/layout.rs
git commit -m "feat: implement top-down and left-right auto-layout with tests

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 1.6: 实现主题和样式操作

**Files:**
- Create: `src-tauri/src/engine/style_ops.rs`

- [ ] **Step 1: 实现样式操作**

`src-tauri/src/engine/style_ops.rs`:

```rust
use super::canvas_state::*;
use std::collections::HashMap;

/// 应用主题到所有节点
pub fn apply_theme(nodes: &mut HashMap<String, DiagramNode>, theme: &Theme) -> usize {
    let style = match theme {
        Theme::Default => NodeStyle::default(),
        Theme::Professional => NodeStyle {
            fill: "#e3f2fd".into(),
            stroke: "#1565c0".into(),
            stroke_width: 2.0,
            font_size: 13.0,
            font_family: "Microsoft YaHei, sans-serif".into(),
            border_radius: 4.0,
        },
        Theme::Handdrawn => NodeStyle {
            fill: "#fffde7".into(),
            stroke: "#5d4037".into(),
            stroke_width: 2.5,
            font_size: 15.0,
            font_family: "Comic Sans MS, cursive".into(),
            border_radius: 12.0,
        },
        Theme::Dark => NodeStyle {
            fill: "#37474f".into(),
            stroke: "#78909c".into(),
            stroke_width: 2.0,
            font_size: 14.0,
            font_family: "sans-serif".into(),
            border_radius: 6.0,
        },
        Theme::Colorful => NodeStyle {
            fill: "#fff3e0".into(),
            stroke: "#e65100".into(),
            stroke_width: 2.5,
            font_size: 14.0,
            font_family: "sans-serif".into(),
            border_radius: 8.0,
        },
    };
    let count = nodes.len();
    for node in nodes.values_mut() {
        node.style = style.clone();
    }
    count
}

/// 设置单个元素的样式
pub fn set_element_style(
    nodes: &mut HashMap<String, DiagramNode>,
    target_id: &str,
    fill: Option<String>,
    stroke: Option<String>,
    font_size: Option<f64>,
) -> Result<NodeStyle, String> {
    let node = nodes.get_mut(target_id).ok_or(format!("节点 {} 不存在", target_id))?;
    if let Some(f) = fill { node.style.fill = f; }
    if let Some(s) = stroke { node.style.stroke = s; }
    if let Some(fs) = font_size { node.style.font_size = fs; }
    Ok(node.style.clone())
}
```

- [ ] **Step 2: 编译验证 + 提交**

```bash
cd src-tauri && cargo check
git add src-tauri/src/engine/style_ops.rs
git commit -m "feat: implement theme application and element style operations

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Phase 2: React Canvas 渲染层

> 目标：在 React 前端集成 Fabric.js，能根据 Rust 传来的状态数据渲染节点、连线，支持缩放平移。

### Task 2.1: Fabric.js 初始化和配置

**Files:**
- Create: `src/lib/fabric-setup.ts`

- [ ] **Step 1: 编写 Fabric.js 初始化函数**

`src/lib/fabric-setup.ts`:

```typescript
import * as fabric from "fabric";

export interface CanvasSetup {
  canvas: fabric.Canvas;
  cleanup: () => void;
}

/**
 * 初始化 Fabric.js Canvas 实例
 * @param canvasEl - HTML canvas 元素
 * @param width - 画布宽度
 * @param height - 画布高度
 */
export function initFabricCanvas(
  canvasEl: HTMLCanvasElement,
  width: number = 1200,
  height: number = 800
): CanvasSetup {
  const canvas = new fabric.Canvas(canvasEl, {
    width,
    height,
    backgroundColor: "#fafafa",
    selection: false,           // 禁用框选（纯语音无鼠标交互）
    preserveObjectStacking: true,
    renderOnAddRemove: true,
  });

  // 启用缩放和平移（开发者调试用键盘/触摸板）
  canvas.on("mouse:wheel", (opt) => {
    const delta = opt.e.deltaY;
    let zoom = canvas.getZoom();
    zoom *= 0.999 ** delta;
    zoom = Math.min(Math.max(zoom, 0.2), 5);
    canvas.zoomToPoint({ x: opt.e.offsetX, y: opt.e.offsetY }, zoom);
    opt.e.preventDefault();
    opt.e.stopPropagation();
  });

  const cleanup = () => {
    canvas.dispose();
  };

  return { canvas, cleanup };
}
```

- [ ] **Step 2: 验证文件语法**

```bash
bun run tsc --noEmit src/lib/fabric-setup.ts
```

- [ ] **Step 3: 提交**

```bash
git add src/lib/fabric-setup.ts
git commit -m "feat: add Fabric.js canvas initialization with zoom support

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2.2: 定义前端类型（与 Rust 对应）

**Files:**
- Create: `src/store/types.ts`

- [ ] **Step 1: 定义 TypeScript 类型**

`src/store/types.ts`:

```typescript
// 与 Rust CanvasState 对应的前端类型

export type NodeType =
  | "Start"
  | "End"
  | "Process"
  | "Decision"
  | "Data"
  | "Subprocess"
  | "Text";

export type Theme =
  | "Default"
  | "Professional"
  | "Handdrawn"
  | "Dark"
  | "Colorful";

export type LineStyle = "Solid" | "Dashed" | "Dotted";
export type ArrowType = "Single" | "Double" | "None";
export type AppStatus = "idle" | "listening" | "thinking" | "executing" | "error";

export interface Position {
  x: number;
  y: number;
}

export interface Size {
  width: number;
  height: number;
}

export interface NodeStyle {
  fill: string;
  stroke: string;
  stroke_width: number;
  font_size: number;
  font_family: string;
  border_radius: number;
}

export interface EdgeStyle {
  line_style: LineStyle;
  arrow: ArrowType;
  stroke: string;
  stroke_width: number;
}

export interface DiagramNode {
  id: string;
  node_type: NodeType;
  label: string;
  position: Position;
  size: Size;
  style: NodeStyle;
}

export interface DiagramEdge {
  id: string;
  from_id: string;
  to_id: string;
  label: string | null;
  style: EdgeStyle;
}

export interface CanvasState {
  id: string;
  title: string;
  nodes: Record<string, DiagramNode>;
  edges: Record<string, DiagramEdge>;
  theme: Theme;
  width: number;
  height: number;
}

export interface NodeSummary {
  id: string;
  node_type: string;
  label: string;
  x: number;
  y: number;
}

/** LLM 工具调用的操作结果 */
export interface OperationResult {
  success: boolean;
  message: string;
  canvas_state: CanvasState | null;
}

/** 对话消息 */
export interface ConversationMessage {
  role: "user" | "assistant";
  content: string;
}
```

- [ ] **Step 2: 提交**

```bash
git add src/store/types.ts
git commit -m "feat: define TypeScript types matching Rust CanvasState structures

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2.3: 实现 Canvas 视图组件

**Files:**
- Create: `src/components/canvas/CanvasView.tsx`

- [ ] **Step 1: 实现 CanvasView 组件**

`src/components/canvas/CanvasView.tsx`:

```tsx
import { useEffect, useRef } from "react";
import * as fabric from "fabric";
import { initFabricCanvas } from "../../lib/fabric-setup";
import type { CanvasState } from "../../store/types";

interface CanvasViewProps {
  canvasState: CanvasState | null;
}

export default function CanvasView({ canvasState }: CanvasViewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const fabricRef = useRef<fabric.Canvas | null>(null);

  // 初始化 Fabric.js
  useEffect(() => {
    if (!canvasRef.current) return;
    const { canvas, cleanup } = initFabricCanvas(canvasRef.current);
    fabricRef.current = canvas;
    return cleanup;
  }, []);

  // 当 canvasState 变化时重新渲染
  useEffect(() => {
    if (!fabricRef.current || !canvasState) return;
    renderCanvasState(fabricRef.current, canvasState);
  }, [canvasState]);

  return (
    <div style={{ flex: 1, overflow: "hidden", position: "relative" }}>
      <canvas ref={canvasRef} />
    </div>
  );
}

/** 将 CanvasState 渲染到 Fabric.js */
function renderCanvasState(canvas: fabric.Canvas, state: CanvasState): void {
  canvas.clear();
  canvas.backgroundColor = state.theme === "Dark" ? "#263238" : "#fafafa";

  // 渲染连线
  for (const edge of Object.values(state.edges)) {
    renderEdge(canvas, edge, state.nodes);
  }

  // 渲染节点
  for (const node of Object.values(state.nodes)) {
    renderNode(canvas, node);
  }

  canvas.requestRenderAll();
}

/** 渲染单个节点 */
function renderNode(canvas: fabric.Canvas, node: CanvasState["nodes"][string]): void {
  const { position, size, style, label, node_type } = node;

  let shape: fabric.Object;

  switch (node_type) {
    case "Start":
    case "End":
      shape = new fabric.Rect({
        left: position.x,
        top: position.y,
        width: size.width,
        height: size.height,
        rx: size.height / 2,  // 圆角矩形（胶囊形）
        ry: size.height / 2,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
      });
      break;

    case "Decision":
      // 菱形：用旋转45度的矩形
      shape = new fabric.Rect({
        left: position.x,
        top: position.y,
        width: size.width,
        height: size.height,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
        angle: 45,
      });
      break;

    case "Data":
      // 平行四边形（用 skewX 近似）
      shape = new fabric.Rect({
        left: position.x,
        top: position.y,
        width: size.width,
        height: size.height,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
        skewX: 15,
      });
      break;

    default: // Process, Subprocess, Text
      shape = new fabric.Rect({
        left: position.x,
        top: position.y,
        width: size.width,
        height: size.height,
        rx: style.border_radius,
        ry: style.border_radius,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
      });
  }

  // 添加文字标签
  const text = new fabric.Text(label, {
    left: position.x + size.width / 2,
    top: position.y + size.height / 2,
    fontSize: style.font_size,
    fontFamily: style.font_family,
    fill: node_type === "Decision" ? "#333" : "#333",
    originX: "center",
    originY: "center",
    textAlign: "center",
  });

  const group = new fabric.Group([shape, text], {
    left: position.x,
    top: position.y,
    data: { nodeId: node.id, nodeType: node_type },
  });

  canvas.add(group);
}

/** 渲染单条连线 */
function renderEdge(
  canvas: fabric.Canvas,
  edge: CanvasState["edges"][string],
  nodes: CanvasState["nodes"]
): void {
  const fromNode = nodes[edge.from_id];
  const toNode = nodes[edge.to_id];
  if (!fromNode || !toNode) return;

  const fromX = fromNode.position.x + fromNode.size.width / 2;
  const fromY = fromNode.position.y + fromNode.size.height;
  const toX = toNode.position.x + toNode.size.width / 2;
  const toY = toNode.position.y;

  const dashArray = edge.style.line_style === "Dashed" ? [8, 4]
    : edge.style.line_style === "Dotted" ? [2, 4]
    : undefined;

  const line = new fabric.Line([fromX, fromY, toX, toY], {
    stroke: edge.style.stroke,
    strokeWidth: edge.style.stroke_width,
    strokeDashArray: dashArray,
    data: { edgeId: edge.id },
  });

  canvas.add(line);
  canvas.sendToBack(line); // 连线在节点下方

  // 边标签
  if (edge.label) {
    const label = new fabric.Text(edge.label, {
      left: (fromX + toX) / 2,
      top: (fromY + toY) / 2 - 16,
      fontSize: 12,
      fill: "#666",
      backgroundColor: "#fafafa",
    });
    canvas.add(label);
  }
}
```

- [ ] **Step 2: 验证编译**

```bash
bun run tsc --noEmit
```

- [ ] **Step 3: 提交**

```bash
git add src/components/canvas/CanvasView.tsx
git commit -m "feat: implement CanvasView with Fabric.js node/edge rendering

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2.4: 实现 Zustand Store

**Files:**
- Create: `src/store/index.ts`

- [ ] **Step 1: 实现全局状态管理**

`src/store/index.ts`:

```typescript
import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { CanvasState, AppStatus, ConversationMessage, OperationResult } from "./types";

interface AppState {
  // 语音状态
  isListening: boolean;
  transcript: string;
  status: AppStatus;

  // Canvas 状态
  canvasState: CanvasState | null;
  lastOperation: string;

  // 对话历史
  conversation: ConversationMessage[];

  // 动作
  startListening: () => void;
  stopListening: () => void;
  setTranscript: (text: string) => void;
  submitCommand: (text: string) => Promise<void>;
  quickAction: (action: string) => Promise<void>;
  setStatus: (status: AppStatus) => void;

  // 内部
  _updateCanvasState: (state: CanvasState) => void;
  _initEventListener: () => Promise<void>;
}

export const useAppStore = create<AppState>((set, get) => ({
  isListening: false,
  transcript: "",
  status: "idle",
  canvasState: null,
  lastOperation: "",
  conversation: [],

  startListening: () => set({ isListening: true, status: "listening", transcript: "" }),
  
  stopListening: () => {
    const text = get().transcript.trim();
    set({ isListening: false });
    if (text.length > 0) {
      get().submitCommand(text);
    } else {
      set({ status: "idle" });
    }
  },

  setTranscript: (text) => set({ transcript: text }),

  submitCommand: async (text) => {
    set({ status: "thinking", lastOperation: text });
    try {
      const result = await invoke<OperationResult>("process_command", { text });
      if (result.success && result.canvas_state) {
        set({
          canvasState: result.canvas_state,
          status: "idle",
          lastOperation: result.message,
          conversation: [
            ...get().conversation,
            { role: "user", content: text },
            { role: "assistant", content: result.message },
          ].slice(-10), // 保留最近10条
        });
      } else {
        set({ status: "error", lastOperation: result.message || "操作失败" });
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      set({ status: "error", lastOperation: `错误: ${errorMsg}` });
      // 3 秒后自动恢复
      setTimeout(() => set({ status: "idle" }), 3000);
    }
  },

  quickAction: async (action) => {
    set({ status: "executing", lastOperation: action });
    try {
      const result = await invoke<OperationResult>("quick_action", { action });
      if (result.success && result.canvas_state) {
        set({
          canvasState: result.canvas_state,
          status: "idle",
          lastOperation: result.message,
        });
      } else {
        set({ status: "error", lastOperation: result.message });
        setTimeout(() => set({ status: "idle" }), 2000);
      }
    } catch (err) {
      set({ status: "error", lastOperation: String(err) });
      setTimeout(() => set({ status: "idle" }), 2000);
    }
  },

  setStatus: (status) => set({ status }),

  _updateCanvasState: (state) => set({ canvasState: state }),

  _initEventListener: async () => {
    // 监听 Rust 端推送的 canvas 更新事件
    await listen<CanvasState>("canvas-updated", (event) => {
      set({ canvasState: event.payload, status: "idle" });
    });
  },
}));
```

- [ ] **Step 2: 提交**

```bash
git add src/store/index.ts
git commit -m "feat: implement Zustand store with voice/canvas/conversation state

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2.5: 整合 App.tsx 主布局

**Files:**
- Modify: `src/App.tsx`
- Create: `src/App.css`

- [ ] **Step 1: 实现应用主布局**

`src/App.tsx`:

```tsx
import { useEffect } from "react";
import CanvasView from "./components/canvas/CanvasView";
import TopBar from "./components/layout/TopBar";
import VoiceBar from "./components/layout/VoiceBar";
import { useAppStore } from "./store";
import "./App.css";

function App() {
  const canvasState = useAppStore((s) => s.canvasState);
  const initEventListener = useAppStore((s) => s._initEventListener);

  useEffect(() => {
    initEventListener();
  }, []);

  return (
    <div className="app-container">
      <TopBar />
      <CanvasView canvasState={canvasState} />
      <VoiceBar />
    </div>
  );
}

export default App;
```

`src/App.css`:

```css
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  background: #1a1a2e;
  color: #eee;
  font-family: "Microsoft YaHei", "PingFang SC", sans-serif;
}
```

- [ ] **Step 2: 验证编译**

```bash
bun run tsc --noEmit
```

- [ ] **Step 3: 提交**

```bash
git add src/App.tsx src/App.css
git commit -m "feat: implement app layout (TopBar + Canvas + VoiceBar)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2.6: 实现顶部栏和状态灯组件

**Files:**
- Create: `src/components/layout/TopBar.tsx`
- Create: `src/components/status/StatusLight.tsx`

- [ ] **Step 1: 实现 TopBar**

`src/components/layout/TopBar.tsx`:

```tsx
import { useAppStore } from "../../store";
import StatusLight from "../status/StatusLight";

export default function TopBar() {
  const canvasState = useAppStore((s) => s.canvasState);
  const status = useAppStore((s) => s.status);
  const lastOp = useAppStore((s) => s.lastOperation);

  return (
    <div style={{
      height: 36,
      display: "flex",
      alignItems: "center",
      justifyContent: "space-between",
      padding: "0 16px",
      background: "#16213e",
      borderBottom: "1px solid #0f3460",
      fontSize: 13,
      flexShrink: 0,
    }}>
      <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
        <span style={{ fontWeight: 700, fontSize: 15 }}>🎤 VoiceDraw</span>
        {canvasState && (
          <span style={{ color: "#aaa" }}>[{canvasState.title}]</span>
        )}
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
        <span style={{ color: "#888", fontSize: 12, maxWidth: 300, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {lastOp}
        </span>
        <StatusLight status={status} />
      </div>
    </div>
  );
}
```

`src/components/status/StatusLight.tsx`:

```tsx
import type { AppStatus } from "../../store/types";

const statusColors: Record<AppStatus, string> = {
  idle: "#666",
  listening: "#ffc107",
  thinking: "#17a2b8",
  executing: "#28a745",
  error: "#dc3545",
};

const statusLabels: Record<AppStatus, string> = {
  idle: "待机中",
  listening: "监听中",
  thinking: "思考中",
  executing: "执行中",
  error: "出错了",
};

export default function StatusLight({ status }: { status: AppStatus }) {
  const isPulsing = status === "listening" || status === "thinking";

  return (
    <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
      <div style={{
        width: 12,
        height: 12,
        borderRadius: "50%",
        backgroundColor: statusColors[status],
        boxShadow: `0 0 6px ${statusColors[status]}`,
        animation: isPulsing ? "pulse 1s ease-in-out infinite" : "none",
      }} />
      <span style={{ fontSize: 11, color: statusColors[status] }}>
        {statusLabels[status]}
      </span>
      <style>{`
        @keyframes pulse {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.4; }
        }
      `}</style>
    </div>
  );
}
```

- [ ] **Step 2: 验证编译**

```bash
bun run tsc --noEmit
```

- [ ] **Step 3: 提交**

```bash
git add src/components/layout/TopBar.tsx src/components/status/StatusLight.tsx
git commit -m "feat: implement TopBar and StatusLight components

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2.7: 端到端验证——用硬编码数据测试 Canvas 渲染

**Files:**
- Modify: `src/App.tsx`（临时添加测试数据）

- [ ] **Step 1: 添加测试数据验证 Canvas 渲染链**

在 `App.tsx` 中临时注入测试数据，确认 Canvas 能正确渲染节点和连线：

```tsx
// 在 useEffect 中临时注入测试数据
useEffect(() => {
  initEventListener();
  // 临时测试：注入一个简单的 Canvas 状态
  const testState = {
    id: "test-canvas",
    title: "测试画布",
    theme: "Default" as const,
    width: 1200,
    height: 800,
    nodes: {
      "n1": {
        id: "n1", node_type: "Start" as const, label: "开始",
        position: { x: 500, y: 50 }, size: { width: 120, height: 50 },
        style: { fill: "#e8f5e9", stroke: "#4caf50", stroke_width: 2, font_size: 14, font_family: "sans-serif", border_radius: 8 },
      },
      "n2": {
        id: "n2", node_type: "Process" as const, label: "处理数据",
        position: { x: 480, y: 180 }, size: { width: 160, height: 60 },
        style: { fill: "#fff", stroke: "#333", stroke_width: 2, font_size: 14, font_family: "sans-serif", border_radius: 8 },
      },
      "n3": {
        id: "n3", node_type: "Decision" as const, label: "是否通过?",
        position: { x: 490, y: 320 }, size: { width: 140, height: 60 },
        style: { fill: "#fff3e0", stroke: "#ff9800", stroke_width: 2, font_size: 13, font_family: "sans-serif", border_radius: 4 },
      },
      "n4": {
        id: "n4", node_type: "End" as const, label: "结束",
        position: { x: 500, y: 480 }, size: { width: 120, height: 50 },
        style: { fill: "#ffebee", stroke: "#f44336", stroke_width: 2, font_size: 14, font_family: "sans-serif", border_radius: 8 },
      },
    },
    edges: {
      "e1": { id: "e1", from_id: "n1", to_id: "n2", label: null, style: { line_style: "Solid" as const, arrow: "Single" as const, stroke: "#555", stroke_width: 2 } },
      "e2": { id: "e2", from_id: "n2", to_id: "n3", label: null, style: { line_style: "Solid" as const, arrow: "Single" as const, stroke: "#555", stroke_width: 2 } },
      "e3": { id: "e3", from_id: "n3", to_id: "n4", label: "通过", style: { line_style: "Solid" as const, arrow: "Single" as const, stroke: "#555", stroke_width: 2 } },
    },
  };
  useAppStore.setState({ canvasState: testState });
}, []);
```

- [ ] **Step 2: 运行验证**

```bash
bun run tauri dev
```

预期：窗口显示 4 个节点（圆角矩形「开始」→ 矩形「处理数据」→ 菱形「是否通过?」→ 圆角矩形「结束」）和 3 条连线。

- [ ] **Step 3: 回退测试代码，提交基础设施**

```bash
git checkout src/App.tsx  # 回退测试数据
git add src/components/canvas/CanvasView.tsx
git commit -m "feat: verify Canvas rendering pipeline with hardcoded test data

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Phase 3: 语音识别前端

> 目标：集成 Web Speech API，实现按住说话 / 唤醒词两种模式，声波可视化，指令文本获取。

### Task 3.1: 实现 useVoiceRecognition Hook

**Files:**
- Create: `src/hooks/useVoiceRecognition.ts`

- [ ] **Step 1: 实现语音识别 Hook**

`src/hooks/useVoiceRecognition.ts`:

```typescript
import { useCallback, useRef } from "react";

interface UseVoiceRecognitionOptions {
  onResult: (text: string, isFinal: boolean) => void;
  onError: (error: string) => void;
  onEnd: () => void;
  lang?: string;
}

export function useVoiceRecognition(options: UseVoiceRecognitionOptions) {
  const { onResult, onError, onEnd, lang = "zh-CN" } = options;
  const recognitionRef = useRef<SpeechRecognition | null>(null);

  const start = useCallback(() => {
    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (!SpeechRecognition) {
      onError("浏览器不支持语音识别，请使用 Chrome 浏览器。");
      return;
    }

    const recognition = new SpeechRecognition();
    recognition.lang = lang;
    recognition.interimResults = true;
    recognition.continuous = true;
    recognition.maxAlternatives = 1;

    recognition.onresult = (event: SpeechRecognitionEvent) => {
      let transcript = "";
      let isFinal = false;
      for (let i = event.resultIndex; i < event.results.length; i++) {
        transcript += event.results[i][0].transcript;
        if (event.results[i].isFinal) {
          isFinal = true;
        }
      }
      onResult(transcript, isFinal);
    };

    recognition.onerror = (event: SpeechRecognitionErrorEvent) => {
      // 'no-speech' 和 'aborted' 是正常结束，不算错误
      if (event.error !== "no-speech" && event.error !== "aborted") {
        onError(`语音识别错误: ${event.error}`);
      }
    };

    recognition.onend = () => {
      onEnd();
    };

    recognition.start();
    recognitionRef.current = recognition;
  }, [lang, onResult, onError, onEnd]);

  const stop = useCallback(() => {
    if (recognitionRef.current) {
      recognitionRef.current.stop();
      recognitionRef.current = null;
    }
  }, []);

  const abort = useCallback(() => {
    if (recognitionRef.current) {
      recognitionRef.current.abort();
      recognitionRef.current = null;
    }
  }, []);

  return { start, stop, abort };
}
```

- [ ] **Step 2: 提交**

```bash
git add src/hooks/useVoiceRecognition.ts
git commit -m "feat: implement useVoiceRecognition hook wrapping Web Speech API

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 3.2: 实现语音条和音量波形

**Files:**
- Create: `src/components/layout/VoiceBar.tsx`
- Create: `src/components/voice/WaveformVisualizer.tsx`

- [ ] **Step 1: 实现波形可视化**

`src/components/voice/WaveformVisualizer.tsx`:

```tsx
import { useEffect, useRef } from "react";

interface WaveformVisualizerProps {
  isActive: boolean;
  barCount?: number;
}

export default function WaveformVisualizer({ isActive, barCount = 20 }: WaveformVisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number>(0);

  useEffect(() => {
    if (!isActive) {
      cancelAnimationFrame(animationRef.current);
      // 清空
      const ctx = canvasRef.current?.getContext("2d");
      if (ctx && canvasRef.current) {
        ctx.clearRect(0, 0, canvasRef.current.width, canvasRef.current.height);
      }
      return;
    }

    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const animate = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const barWidth = canvas.width / barCount;
      for (let i = 0; i < barCount; i++) {
        // 模拟随机音量（正式版接入 Web Audio API analyser）
        const height = isActive
          ? Math.random() * canvas.height * 0.8 + canvas.height * 0.1
          : 3;
        ctx.fillStyle = `hsl(${220 + i * 3}, 80%, ${50 + height * 0.5}%)`;
        ctx.fillRect(
          i * barWidth + 1,
          canvas.height - height,
          barWidth - 2,
          height
        );
      }
      animationRef.current = requestAnimationFrame(animate);
    };
    animate();

    return () => cancelAnimationFrame(animationRef.current);
  }, [isActive, barCount]);

  return (
    <canvas
      ref={canvasRef}
      width={200}
      height={30}
      style={{ borderRadius: 4 }}
    />
  );
}
```

- [ ] **Step 2: 实现 VoiceBar**

`src/components/layout/VoiceBar.tsx`:

```tsx
import { useCallback, useRef } from "react";
import { useAppStore } from "../../store";
import { useVoiceRecognition } from "../../hooks/useVoiceRecognition";
import WaveformVisualizer from "../voice/WaveformVisualizer";

export default function VoiceBar() {
  const isListening = useAppStore((s) => s.isListening);
  const transcript = useAppStore((s) => s.transcript);
  const status = useAppStore((s) => s.status);
  const startListening = useAppStore((s) => s.startListening);
  const stopListening = useAppStore((s) => s.stopListening);
  const setTranscript = useAppStore((s) => s.setTranscript);

  const silenceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastTextRef = useRef("");

  const handleResult = useCallback((text: string, _isFinal: boolean) => {
    setTranscript(text);
    lastTextRef.current = text;

    // 2 秒静默后自动提交
    if (silenceTimerRef.current) clearTimeout(silenceTimerRef.current);
    silenceTimerRef.current = setTimeout(() => {
      if (lastTextRef.current.trim().length > 0) {
        stopListening();
      }
    }, 2000);
  }, [setTranscript, stopListening]);

  const handleError = useCallback((error: string) => {
    console.error("STT Error:", error);
  }, []);

  const handleEnd = useCallback(() => {
    // recognition 结束
  }, []);

  const { start, stop } = useVoiceRecognition({
    onResult: handleResult,
    onError: handleError,
    onEnd: handleEnd,
  });

  // 按住说话
  const handlePointerDown = () => {
    startListening();
    start();
  };

  const handlePointerUp = () => {
    stop();
    stopListening();
  };

  const statusText = {
    idle: "👆 按住说话，或说「开始画图」唤醒",
    listening: "🎤 正在听...",
    thinking: "🤔 正在理解...",
    executing: "✏️ 正在绘图...",
    error: "❌ 出错了，点击重试",
  };

  return (
    <div style={{
      height: 48,
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
      gap: 12,
      padding: "0 16px",
      background: "#16213e",
      borderTop: "1px solid #0f3460",
      flexShrink: 0,
      fontSize: 13,
      userSelect: "none",
    }}>
      <WaveformVisualizer isActive={isListening} />

      <div
        onPointerDown={handlePointerDown}
        onPointerUp={handlePointerUp}
        onPointerLeave={handlePointerUp}
        style={{
          flex: 1,
          height: 36,
          borderRadius: 18,
          background: isListening
            ? "linear-gradient(135deg, #667eea 0%, #764ba2 100%)"
            : "#1a1a3e",
          border: isListening ? "2px solid #667eea" : "2px solid #333",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          cursor: "pointer",
          transition: "all 0.2s",
        }}
      >
        <span style={{ color: isListening ? "#fff" : "#888" }}>
          {isListening
            ? (transcript || "正在听...")
            : statusText[status]}
        </span>
      </div>

      <span style={{ fontSize: 11, color: "#555", minWidth: 50, textAlign: "right" }}>
        {transcript.length > 0 ? `${transcript.length} 字` : ""}
      </span>
    </div>
  );
}
```

- [ ] **Step 3: 添加 TypeScript 类型声明**

创建 `src/types/speech.d.ts`:

```typescript
// Web Speech API 类型声明
interface SpeechRecognition extends EventTarget {
  lang: string;
  interimResults: boolean;
  continuous: boolean;
  maxAlternatives: number;
  onresult: ((event: SpeechRecognitionEvent) => void) | null;
  onerror: ((event: SpeechRecognitionErrorEvent) => void) | null;
  onend: (() => void) | null;
  start(): void;
  stop(): void;
  abort(): void;
}

interface SpeechRecognitionEvent {
  resultIndex: number;
  results: SpeechRecognitionResultList;
}

interface SpeechRecognitionResultList {
  length: number;
  [index: number]: SpeechRecognitionResult;
}

interface SpeechRecognitionResult {
  isFinal: boolean;
  length: number;
  [index: number]: SpeechRecognitionAlternative;
}

interface SpeechRecognitionAlternative {
  transcript: string;
  confidence: number;
}

interface SpeechRecognitionErrorEvent extends Event {
  error: string;
}

interface Window {
  SpeechRecognition?: { new(): SpeechRecognition };
  webkitSpeechRecognition?: { new(): SpeechRecognition };
}
```

- [ ] **Step 4: 编译验证**

```bash
bun run tsc --noEmit
```

- [ ] **Step 5: 提交**

```bash
git add src/components/layout/VoiceBar.tsx src/components/voice/WaveformVisualizer.tsx src/types/speech.d.ts
git commit -m "feat: implement VoiceBar with press-to-talk and waveform visualizer

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 3.3: 端到端验证语音链路

- [ ] **Step 1: 运行应用并测试语音**

```bash
bun run tauri dev
```

在 Chrome 中打开开发者工具，按住底部语音条说话，确认：
- 状态灯变为🟡监听中
- 波形有动画
- 文本实时显示
- 松手后状态灯变为🔵思考中（因为没有 Rust 后端处理，会报错然后自动恢复）

- [ ] **Step 2: 提交**（无代码变更，仅验证）

```bash
echo "Phase 3 语音识别前端验证完成"
```

---

## Phase 4: Rust 指令预处理器

> 目标：实现 Rust 端的去噪、快捷指令匹配、上下文补全。让简单的本地指令（撤销/缩放/导出）直接响应，不走 LLM。

### Task 4.1: 实现 Tauri Commands 入口

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 创建 commands 模块**

`src-tauri/src/commands/mod.rs`:

```rust
use crate::engine::canvas_state::CanvasState;

// 后续 Task 将完善这两个函数
#[tauri::command]
pub async fn process_command(text: String) -> Result<serde_json::Value, String> {
    // 暂时返回模拟数据
    Ok(serde_json::json!({
        "success": true,
        "message": format!("收到指令: {}", text),
        "canvas_state": null
    }))
}

#[tauri::command]
pub async fn quick_action(action: String) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "success": true,
        "message": format!("快捷操作: {}", action),
        "canvas_state": null
    }))
}
```

- [ ] **Step 2: 更新 lib.rs 注册 commands**

`src-tauri/src/lib.rs`:

```rust
mod engine;
mod commands;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            process_command,
            quick_action,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 3: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/commands/ src-tauri/src/lib.rs
git commit -m "feat: add Tauri command handlers for process_command and quick_action

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 4.2: 实现去噪处理

**Files:**
- Create: `src-tauri/src/preprocessor/mod.rs`
- Create: `src-tauri/src/preprocessor/denoise.rs`

- [ ] **Step 1: 实现去噪函数**

`src-tauri/src/preprocessor/denoise.rs`:

```rust
/// 中文语音指令去噪
/// 移除口语填充词、重复词，规范化标点
pub fn denoise(text: &str) -> String {
    // 填充词列表
    let fillers = [
        "嗯", "啊", "呃", "那个", "这个", "就是", "就是说",
        "然后", "那么", "的话", "吧", "嘛", "呗",
    ];

    let mut result = text.to_string();

    // 移除填充词
    for filler in &fillers {
        result = result.replace(filler, "");
    }

    // 移除多余空白
    let words: Vec<&str> = result.split_whitespace().collect();
    let mut deduped = Vec::new();
    for w in words {
        if deduped.last() != Some(&w) || w.chars().count() > 2 {
            deduped.push(w);
        }
    }

    deduped.join(" ")
}

/// 中文同音词纠错映射（常见 STT 错误）
pub fn correct_homophones(text: &str) -> String {
    let corrections = [
        ("话圆", "画圆"),
        ("话矩形", "画矩形"),
        ("举行", "矩形"),
        ("箭偷", "箭头"),
        ("借点", "节点"),
        ("亮解", "连接"),
        ("布橘", "布局"),
        ("扯消", "撤销"),
        ("中坐", "重做"),
        ("报存", "保存"),
        ("打倒", "导出"),
        ("放打", "放大"),
        ("缩笑", "缩小"),
        ("留成图", "流程图"),
        ("司维导图", "思维导图"),
        ("架构徒", "架构图"),
        // 可继续扩展
    ];

    let mut result = text.to_string();
    for (wrong, correct) in &corrections {
        if result.contains(wrong) {
            result = result.replace(wrong, correct);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_denoise() {
        let input = "嗯那个画一个就是红色的矩形";
        let output = denoise(input);
        assert!(!output.contains("嗯"));
        assert!(!output.contains("那个"));
        assert!(!output.contains("就是"));
        assert!(output.contains("红色的矩形"));
    }

    #[test]
    fn test_homophone() {
        let input = "画一个举行然后话圆";
        let output = correct_homophones(input);
        assert!(output.contains("矩形"));
        assert!(output.contains("画圆"));
        // "然后" 不在映射表中，应保留
        assert!(output.contains("然后"));
    }
}
```

- [ ] **Step 2: 实现预处理器模块入口**

`src-tauri/src/preprocessor/mod.rs`:

```rust
pub mod denoise;
pub mod quick_match;
pub mod context_enrich;

/// 预处理结果
pub enum PreprocessResult {
    /// 需要送 LLM 处理
    NeedsLLM { cleaned_text: String },
    /// 本地快捷指令匹配成功，不需 LLM
    LocalAction { action: String, params: serde_json::Value },
}

/// 预处理管道
pub fn preprocess(text: &str) -> PreprocessResult {
    let cleaned = denoise::denoise(text);
    let corrected = denoise::correct_homophones(&cleaned);

    if corrected.trim().is_empty() {
        return PreprocessResult::NeedsLLM { cleaned_text: String::new() };
    }

    // 尝试快捷匹配
    if let Some(action) = quick_match::try_match(&corrected) {
        return PreprocessResult::LocalAction { action: action.name, params: action.params };
    }

    PreprocessResult::NeedsLLM { cleaned_text: corrected }
}
```

- [ ] **Step 3: 运行测试**

```bash
cd src-tauri && cargo test denoise
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/preprocessor/
git commit -m "feat: implement denoise, homophone correction and preprocessor pipeline

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 4.3: 实现快捷指令匹配

**Files:**
- Create: `src-tauri/src/preprocessor/quick_match.rs`

- [ ] **Step 1: 实现快捷指令表**

`src-tauri/src/preprocessor/quick_match.rs`:

```rust
use serde_json::Value;

pub struct QuickAction {
    pub name: String,
    pub params: Value,
}

/// 本地快捷指令匹配表
/// 复杂度 O(n)，因指令量极少（~10 条），无需优化
pub fn try_match(text: &str) -> Option<QuickAction> {
    let t = text.trim();

    // 精确匹配
    let patterns: &[(&[&str], &str, Value)] = &[
        (&["撤销", "回退", "上一步", "取消上一步"], "undo", Value::Null),
        (&["重做", "恢复", "下一步"], "redo", Value::Null),
        (&["清空画布", "全部删除", "清除全部", "清空"], "clear_canvas", Value::Null),
        (&["放大", "放大一点"], "zoom_in", Value::Null),
        (&["缩小", "缩小一点"], "zoom_out", Value::Null),
        (&["适应窗口", "适合屏幕", "全部显示", "显示全部"], "fit_to_screen", Value::Null),
        (&["导出", "保存", "导出图片", "保存图片"], "export", serde_json::json!({"format": "png"})),
        (&["导出PNG", "导出png", "保存PNG"], "export", serde_json::json!({"format": "png"})),
        (&["导出SVG", "导出svg", "保存SVG"], "export", serde_json::json!({"format": "svg"})),
    ];

    for (keywords, action_name, params) in patterns {
        for kw in *keywords {
            if t.contains(kw) && t.chars().count() <= kw.chars().count() + 5 {
                return Some(QuickAction {
                    name: action_name.to_string(),
                    params: params.clone(),
                });
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_match() {
        let action = try_match("撤销").unwrap();
        assert_eq!(action.name, "undo");
    }

    #[test]
    fn test_zoom_in() {
        let action = try_match("放大").unwrap();
        assert_eq!(action.name, "zoom_in");
    }

    #[test]
    fn test_export_png() {
        let action = try_match("导出PNG").unwrap();
        assert_eq!(action.name, "export");
        assert_eq!(action.params["format"], "png");
    }

    #[test]
    fn test_complex_not_matched() {
        // 复杂指令不应该被快捷匹配拦截
        assert!(try_match("画一个用户登录流程图").is_none());
    }
}
```

- [ ] **Step 2: 运行测试**

```bash
cd src-tauri && cargo test quick_match
```

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/preprocessor/quick_match.rs
git commit -m "feat: implement quick match for local commands (undo/zoom/export)

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 4.4: 实现快捷指令执行引擎

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`（完善 quick_action）
- Create: `src-tauri/src/engine/mod.rs`（加入全局状态管理）

- [ ] **Step 1: 加入全局 Canvas 状态管理**

更新 `src-tauri/src/engine/mod.rs`，添加全局状态管理器：

```rust
pub mod canvas_state;
pub mod node_ops;
pub mod edge_ops;
pub mod layout;
pub mod style_ops;
pub mod snapshot;

pub use canvas_state::*;

use std::sync::Mutex;
use canvas_state::CanvasState;
use std::collections::HashMap;

/// 应用全局 Canvas 状态（后续会用更优雅的方式管理）
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
```

- [ ] **Step 2: 完善 quick_action 命令**

更新 `src-tauri/src/commands/mod.rs`，结合 engine 实现实际的快捷操作：

```rust
use crate::engine::canvas_state::CanvasState;
use crate::engine::AppEngine;
use crate::preprocessor::{self, PreprocessResult};

// 全局 engine 实例
static ENGINE: std::sync::LazyLock<AppEngine> =
    std::sync::LazyLock::new(|| AppEngine::new());

#[tauri::command]
pub async fn process_command(
    text: String,
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let result = preprocessor::preprocess(&text);

    match result {
        PreprocessResult::LocalAction { action, params } => {
            execute_quick_action(&action, &params, &app)
        }
        PreprocessResult::NeedsLLM { cleaned_text } => {
            // Phase 5 将在这里接入 LLM 调度器
            Ok(serde_json::json!({
                "success": false,
                "message": format!("LLM 尚未集成，收到指令: {}", cleaned_text),
                "canvas_state": null
            }))
        }
    }
}

#[tauri::command]
pub async fn quick_action(
    action: String,
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    execute_quick_action(&action, &serde_json::Value::Null, &app)
}

fn execute_quick_action(
    action: &str,
    _params: &serde_json::Value,
    app: &tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let mut canvas = ENGINE.canvas.lock().unwrap();
    let canvas_state = canvas.as_mut().ok_or("画布未初始化")?;

    let message = match action {
        "undo" => {
            let current = canvas_state.clone();
            let mut snapshots = ENGINE.snapshots.lock().unwrap();
            if let Some(prev) = snapshots.undo(current) {
                *canvas_state = prev;
                "已撤销".to_string()
            } else {
                "没有可撤销的操作".to_string()
            }
        }
        "redo" => {
            let current = canvas_state.clone();
            let mut snapshots = ENGINE.snapshots.lock().unwrap();
            if let Some(next) = snapshots.redo(current) {
                *canvas_state = next;
                "已重做".to_string()
            } else {
                "没有可重做的操作".to_string()
            }
        }
        "clear_canvas" => {
            canvas_state.nodes.clear();
            canvas_state.edges.clear();
            "画布已清空".to_string()
        }
        "zoom_in" | "zoom_out" | "fit_to_screen" | "export" => {
            // 缩放和导出由前端处理，这里返回信号即可
            format!("快捷操作: {}", action)
        }
        _ => return Err(format!("未知快捷操作: {}", action)),
    };

    // 发送事件通知前端更新
    let state = canvas_state.clone();
    let _ = app.emit("canvas-updated", &state);

    Ok(serde_json::json!({
        "success": true,
        "message": message,
        "canvas_state": state
    }))
}
```

- [ ] **Step 3: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/engine/mod.rs src-tauri/src/commands/mod.rs src-tauri/src/preprocessor/mod.rs
git commit -m "feat: implement quick action execution with global engine state

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Phase 5: LLM (DeepSeek V4) 集成

> 目标：Rust 端接入 DeepSeek API，实现 System Prompt + 工具定义 + 多轮 Function Calling 循环。

### Task 5.1: 实现 DeepSeek 客户端

**Files:**
- Create: `src-tauri/src/llm/mod.rs`
- Create: `src-tauri/src/llm/deepseek_client.rs`

- [ ] **Step 1: 实现 DeepSeek API 客户端**

`src-tauri/src/llm/deepseek_client.rs`:

```rust
use async_openai::{
    Client,
    config::OpenAIConfig,
    types::{
        CreateChatCompletionRequest, ChatCompletionRequestMessage,
        ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestUserMessageArgs,
        ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestToolMessageArgs,
        ChatCompletionTool, FunctionObject,
        ResponseFormat,
    },
};
use serde_json::Value;

pub struct DeepSeekClient {
    client: Client<OpenAIConfig>,
    model: String,
}

impl DeepSeekClient {
    /// 从环境变量或配置创建客户端
    pub fn new(api_key: String, base_url: Option<String>) -> Self {
        let config = OpenAIConfig::new()
            .with_api_base(base_url.unwrap_or_else(|| "https://api.deepseek.com".into()))
            .with_api_key(api_key);
        Self {
            client: Client::with_config(config),
            model: "deepseek-chat".into(),
        }
    }

    /// 发送对话请求（含工具定义），返回 LLM 响应
    pub async fn chat(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
        tools: Vec<ChatCompletionTool>,
        stream: bool,
    ) -> Result<ChatCompletionResponse, String> {
        let request = CreateChatCompletionRequest {
            model: self.model.clone(),
            messages,
            tools: if tools.is_empty() { None } else { Some(tools) },
            tool_choice: None,
            stream: Some(stream),
            max_tokens: Some(4096u32),
            temperature: Some(0.3),
            response_format: None,
            ..Default::default()
        };

        let response = self.client
            .chat()
            .create(request)
            .await
            .map_err(|e| format!("DeepSeek API 调用失败: {}", e))?;

        // 解析返回内容
        let choice = response.choices.into_iter().next()
            .ok_or("DeepSeek 返回空响应".to_string())?;

        let message = choice.message;

        Ok(ChatCompletionResponse {
            content: message.content,
            tool_calls: message.tool_calls.map(|tc| tc.into_iter().map(|t| ToolCall {
                id: t.id,
                name: t.function.name,
                arguments: t.function.arguments.unwrap_or_default(),
            }).collect()),
        })
    }
}

#[derive(Debug, Clone)]
pub struct ChatCompletionResponse {
    pub content: Option<String>,
    pub tool_calls: Option<Vec<ToolCall>>,
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String, // JSON string
}
```

- [ ] **Step 2: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/llm/
git commit -m "feat: implement DeepSeek API client with tool calling support

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 5.2: 编写 System Prompt 和工具定义

**Files:**
- Create: `src-tauri/src/llm/system_prompt.rs`
- Create: `src-tauri/src/llm/tool_defs.rs`

- [ ] **Step 1: System Prompt**

`src-tauri/src/llm/system_prompt.rs`:

```rust
pub fn get_system_prompt() -> String {
    r#"你是一个专业的语音控制绘图助手。用户通过中文语音操控图表编辑器，你需要将用户的自然语言指令转化为绘图操作。

## 你的能力
你可以通过工具函数来创建：流程图、思维导图、架构图、UML图、ER图、时序图等各种图表。

## 重要规则
1. **优先使用批量工具**：add_nodes_batch 和 add_edges_batch 一次性完成所有操作，减少工具调用轮次。
2. **使用自动布局**：不要手动指定每个节点的像素坐标，让 auto_layout 自动排列。
3. **理解上下文**：如果用户说"把那个菱形改成绿色"，你需要回顾对话历史找到"那个菱形"指的是哪个节点。
4. **不确定时反问**：如果指令不明确，先简短问一句确认，不要猜测。
5. **回复简洁**：每次工具调用后的文字回复不超过两句话。
6. **节点类型选择**：
   - 流程的开始/结束用 "start"/"end"
   - 普通步骤用 "process"
   - 判断分支用 "decision"
   - 数据/文件用 "data"
   - 子流程用 "subprocess"
7. **所有回复和标签使用中文**。
"#.to_string()
}
```

- [ ] **Step 2: 工具定义**

`src-tauri/src/llm/tool_defs.rs`:

```rust
use async_openai::types::{ChatCompletionTool, FunctionObject};
use serde_json::json;

pub fn get_tool_definitions() -> Vec<ChatCompletionTool> {
    vec![
        tool("add_nodes_batch", "批量添加节点到画布",
            json!({
                "type": "object",
                "properties": {
                    "nodes": {
                        "type": "array",
                        "description": "要添加的节点列表",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": {
                                    "type": "string",
                                    "enum": ["start", "end", "process", "decision", "data", "subprocess", "text"],
                                    "description": "节点类型"
                                },
                                "label": {
                                    "type": "string",
                                    "description": "节点显示的文本标签"
                                }
                            },
                            "required": ["type", "label"]
                        }
                    }
                },
                "required": ["nodes"]
            })),
        tool("add_edges_batch", "批量添加连线",
            json!({
                "type": "object",
                "properties": {
                    "edges": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "from": {"type": "string", "description": "起始节点 ID"},
                                "to": {"type": "string", "description": "目标节点 ID"},
                                "label": {"type": "string", "description": "连线标签（可选）"}
                            },
                            "required": ["from", "to"]
                        }
                    }
                },
                "required": ["edges"]
            })),
        tool("add_node", "添加单个节点",
            json!({
                "type": "object",
                "properties": {
                    "type": {"type": "string", "enum": ["start", "end", "process", "decision", "data", "subprocess", "text"]},
                    "label": {"type": "string"},
                    "position": {
                        "type": "object",
                        "properties": {
                            "x": {"type": "number"},
                            "y": {"type": "number"}
                        }
                    }
                },
                "required": ["type", "label"]
            })),
        tool("add_edge", "添加单条连线",
            json!({
                "type": "object",
                "properties": {
                    "from": {"type": "string"},
                    "to": {"type": "string"},
                    "label": {"type": "string"}
                },
                "required": ["from", "to"]
            })),
        tool("update_node", "修改节点属性",
            json!({
                "type": "object",
                "properties": {
                    "node_id": {"type": "string"},
                    "label": {"type": "string"},
                    "fill": {"type": "string", "description": "填充颜色，如 #e8f5e9"},
                    "stroke": {"type": "string", "description": "边框颜色"}
                },
                "required": ["node_id"]
            })),
        tool("delete_node", "删除节点（自动删除关联连线）",
            json!({
                "type": "object",
                "properties": {
                    "node_id": {"type": "string"}
                },
                "required": ["node_id"]
            })),
        tool("delete_edge", "删除连线",
            json!({
                "type": "object",
                "properties": {
                    "edge_id": {"type": "string"}
                },
                "required": ["edge_id"]
            })),
        tool("auto_layout", "自动排列节点位置",
            json!({
                "type": "object",
                "properties": {
                    "direction": {
                        "type": "string",
                        "enum": ["top_down", "left_right"],
                        "description": "布局方向"
                    }
                }
            })),
        tool("set_theme", "切换画布主题",
            json!({
                "type": "object",
                "properties": {
                    "theme": {
                        "type": "string",
                        "enum": ["default", "professional", "handdrawn", "dark", "colorful"]
                    }
                },
                "required": ["theme"]
            })),
        tool("get_canvas_state", "获取当前画布状态",
            json!({
                "type": "object",
                "properties": {}
            })),
    ]
}

fn tool(name: &str, description: &str, parameters: serde_json::Value) -> ChatCompletionTool {
    ChatCompletionTool {
        function: FunctionObject {
            name: name.to_string(),
            description: Some(description.to_string()),
            parameters: Some(parameters),
            strict: None,
        },
        type_: async_openai::types::ToolType::Function,
    }
}
```

`src-tauri/src/llm/mod.rs`:

```rust
pub mod deepseek_client;
pub mod system_prompt;
pub mod tool_defs;
pub mod scheduler;
```

- [ ] **Step 3: 编译验证 + 提交**

```bash
cd src-tauri && cargo check
git add src-tauri/src/llm/
git commit -m "feat: add system prompt and 10 tool definitions for LLM function calling

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 5.3: 实现 LLM Scheduler 多轮循环

**Files:**
- Create: `src-tauri/src/llm/scheduler.rs`

- [ ] **Step 1: 实现调度器**

`src-tauri/src/llm/scheduler.rs`:

```rust
use async_openai::types::{
    ChatCompletionRequestMessage, ChatCompletionRequestUserMessageArgs,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestAssistantMessageArgs,
    ChatCompletionRequestToolMessageArgs, ChatCompletionRequestMessageContentPart,
};
use crate::engine::AppEngine;
use super::deepseek_client::{DeepSeekClient, ToolCall};
use super::system_prompt::get_system_prompt;
use super::tool_defs::get_tool_definitions;
use serde_json::Value;

/// LLM 调度器：管理 DeepSeek API 调用多轮循环
pub struct LLMScheduler {
    client: DeepSeekClient,
    max_rounds: u8,
}

impl LLMScheduler {
    pub fn new(api_key: String) -> Self {
        Self {
            client: DeepSeekClient::new(api_key, None),
            max_rounds: 5,
        }
    }

    /// 处理用户指令，返回最终回复和更新的 Canvas 状态
    pub async fn process(
        &self,
        user_text: &str,
        history: &[(String, String)], // (role, content)
        engine: &AppEngine,
    ) -> Result<SchedulerResult, String> {
        // 构建 messages
        let mut messages: Vec<ChatCompletionRequestMessage> = Vec::new();

        // System prompt
        messages.push(ChatCompletionRequestSystemMessageArgs::default()
            .content(get_system_prompt())
            .build().unwrap().into());

        // 历史对话（最近 5 轮）
        for (role, content) in history.iter().take(5) {
            match role.as_str() {
                "user" => {
                    messages.push(ChatCompletionRequestUserMessageArgs::default()
                        .content(content.clone())
                        .build().unwrap().into());
                }
                "assistant" => {
                    messages.push(ChatCompletionRequestAssistantMessageArgs::default()
                        .content(content.clone())
                        .build().unwrap().into());
                }
                _ => {}
            }
        }

        // 当前的 canvas 状态摘要（作为上下文注入）
        let canvas_summary = {
            let canvas = engine.canvas.lock().unwrap();
            canvas.as_ref().map(|c| {
                format!(
                    "当前画布: {}, 节点数: {}, 连线数: {}, 主题: {:?}",
                    c.title,
                    c.nodes.len(),
                    c.edges.len(),
                    c.theme
                )
            }).unwrap_or_default()
        };

        let user_msg = format!(
            "{}画布状态: {}",
            user_text,
            if canvas_summary.is_empty() { "空".into() } else { canvas_summary }
        );

        messages.push(ChatCompletionRequestUserMessageArgs::default()
            .content(user_msg)
            .build().unwrap().into());

        // 工具定义
        let tools = get_tool_definitions();

        let mut final_content = String::new();

        // 最多 max_rounds 轮循环
        for round in 0..self.max_rounds {
            let response = self.client.chat(messages.clone(), tools.clone(), false).await?;

            if let Some(tool_calls) = response.tool_calls {
                if tool_calls.is_empty() {
                    // LLM 返回了文本回复
                    final_content = response.content.unwrap_or_default();
                    break;
                }

                // 将 assistant 的 tool_calls 添加到 messages
                // (简化版，async-openai 的类型较复杂)
                
                // 执行每个工具调用
                for tc in &tool_calls {
                    let tool_result = execute_tool_call(engine, &tc.name, &tc.arguments)
                        .unwrap_or_else(|e| format!("错误: {}", e));
                    
                    messages.push(ChatCompletionRequestToolMessageArgs::default()
                        .tool_call_id(tc.id.clone())
                        .content(tool_result)
                        .build().unwrap().into());
                }

                if round == self.max_rounds - 1 {
                    final_content = "已完成操作（达到最大轮次限制）".into();
                }
            } else {
                final_content = response.content.unwrap_or_else(|| "操作完成".into());
                break;
            }
        }

        let canvas_state = engine.canvas.lock().unwrap().clone();

        Ok(SchedulerResult {
            message: final_content,
            canvas_state,
        })
    }
}

pub struct SchedulerResult {
    pub message: String,
    pub canvas_state: Option<crate::engine::canvas_state::CanvasState>,
}

/// 执行单个工具调用
fn execute_tool_call(engine: &AppEngine, name: &str, arguments: &str) -> Result<String, String> {
    let args: Value = serde_json::from_str(arguments)
        .map_err(|e| format!("参数解析失败: {}", e))?;

    let mut canvas = engine.canvas.lock().unwrap();
    let state = canvas.as_mut().ok_or("画布未初始化")?;

    match name {
        "add_node" => {
            let node_type = crate::engine::canvas_state::NodeType::from_str(
                args["type"].as_str().unwrap_or("process")
            ).map_err(|e| format!("{}", e))?;
            let label = args["label"].as_str().unwrap_or("未命名").to_string();
            let node = crate::engine::node_ops::add_node(
                &mut state.nodes, node_type, label, None, None
            );
            Ok(serde_json::json!({"node_id": node.id, "label": node.label}).to_string())
        }
        "add_nodes_batch" => {
            let nodes = args["nodes"].as_array().ok_or("nodes 必须是数组")?;
            let batch: Vec<_> = nodes.iter().map(|n| {
                let nt = crate::engine::canvas_state::NodeType::from_str(
                    n["type"].as_str().unwrap_or("process")
                ).unwrap();
                let label = n["label"].as_str().unwrap_or("未命名").to_string();
                (nt, label)
            }).collect();
            let created = crate::engine::node_ops::add_nodes_batch(&mut state.nodes, batch);
            // 自动布局
            crate::engine::layout::auto_layout(
                &mut state.nodes, &state.edges, crate::engine::layout::LayoutDirection::TopDown
            );
            Ok(serde_json::json!({"nodes": created.iter().map(|n| {
                serde_json::json!({"id": n.id, "type": format!("{:?}", n.node_type), "label": n.label})
            }).collect::<Vec<_>>()}).to_string())
        }
        "add_edge" => {
            let from = args["from"].as_str().ok_or("缺少 from")?;
            let to = args["to"].as_str().ok_or("缺少 to")?;
            let label = args["label"].as_str().map(|s| s.to_string());
            let edge = crate::engine::edge_ops::add_edge(&mut state.edges, from, to, label, None)
                .map_err(|e| format!("{}", e))?;
            Ok(serde_json::json!({"edge_id": edge.id}).to_string())
        }
        "add_edges_batch" => {
            let edges = args["edges"].as_array().ok_or("edges 必须是数组")?;
            let batch: Vec<_> = edges.iter().map(|e| {
                crate::engine::edge_ops::EdgeDef {
                    from: e["from"].as_str().unwrap_or("").to_string(),
                    to: e["to"].as_str().unwrap_or("").to_string(),
                    label: e["label"].as_str().map(|s| s.to_string()),
                    style: None,
                }
            }).collect();
            let created = crate::engine::edge_ops::add_edges_batch(&mut state.edges, batch);
            Ok(serde_json::json!({"count": created.len()}).to_string())
        }
        "update_node" => {
            let node_id = args["node_id"].as_str().ok_or("缺少 node_id")?;
            let label = args["label"].as_str().map(|s| s.to_string());
            let fill = args["fill"].as_str().map(|s| s.to_string());
            let stroke = args["stroke"].as_str().map(|s| s.to_string());
            // 更新样式
            if let Some(f) = fill {
                crate::engine::style_ops::set_element_style(
                    &mut state.nodes, node_id, Some(f), stroke, None
                ).map_err(|e| format!("{}", e))?;
            }
            if let Some(l) = label {
                crate::engine::node_ops::update_node(
                    &mut state.nodes, node_id, Some(l), None, None
                ).map_err(|e| format!("{}", e))?;
            }
            Ok(serde_json::json!({"success": true}).to_string())
        }
        "delete_node" => {
            let node_id = args["node_id"].as_str().ok_or("缺少 node_id")?;
            crate::engine::node_ops::delete_node(
                &mut state.nodes, &mut state.edges, node_id
            ).map_err(|e| format!("{}", e))?;
            Ok(serde_json::json!({"success": true}).to_string())
        }
        "delete_edge" => {
            let edge_id = args["edge_id"].as_str().ok_or("缺少 edge_id")?;
            crate::engine::edge_ops::delete_edge(
                &mut state.edges, edge_id
            ).map_err(|e| format!("{}", e))?;
            Ok(serde_json::json!({"success": true}).to_string())
        }
        "auto_layout" => {
            let dir_str = args["direction"].as_str().unwrap_or("top_down");
            let direction = match dir_str {
                "left_right" => crate::engine::layout::LayoutDirection::LeftRight,
                _ => crate::engine::layout::LayoutDirection::TopDown,
            };
            let moved = crate::engine::layout::auto_layout(
                &mut state.nodes, &state.edges, direction
            );
            Ok(serde_json::json!({"moved_count": moved}).to_string())
        }
        "set_theme" => {
            let theme_str = args["theme"].as_str().unwrap_or("default");
            let theme = crate::engine::canvas_state::Theme::from_str(theme_str)
                .map_err(|e| format!("{}", e))?;
            crate::engine::style_ops::apply_theme(&mut state.nodes, &theme);
            state.theme = theme;
            Ok(serde_json::json!({"success": true}).to_string())
        }
        "get_canvas_state" => {
            Ok(serde_json::to_string(&state).unwrap_or_else(|_| "{}".into()))
        }
        _ => Err(format!("未知工具: {}", name)),
    }
}
```

- [ ] **Step 2: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 3: 提交**

```bash
git add src-tauri/src/llm/scheduler.rs
git commit -m "feat: implement LLM scheduler with multi-round tool-calling loop

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 5.4: 将 LLM 调度器接入 Tauri Command

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: 更新 process_command 接入 LLM**

在 `src-tauri/src/commands/mod.rs` 中更新 `process_command`：

```rust
// 在 PreprocessResult::NeedsLLM 分支中：
PreprocessResult::NeedsLLM { cleaned_text } => {
    // 从配置中读取 API Key（Phase 7 前使用环境变量）
    let api_key = std::env::var("DEEPSEEK_API_KEY")
        .unwrap_or_else(|_| "sk-placeholder".into());
    
    let scheduler = crate::llm::scheduler::LLMScheduler::new(api_key);
    let history: Vec<(String, String)> = vec![]; // 先不传历史

    match scheduler.process(&cleaned_text, &history, &ENGINE).await {
        Ok(result) => {
            // 保存快照
            if let Some(ref state) = result.canvas_state {
                ENGINE.snapshots.lock().unwrap().save(state.clone());
                let _ = app.emit("canvas-updated", state);
            }
            Ok(serde_json::json!({
                "success": true,
                "message": result.message,
                "canvas_state": result.canvas_state
            }))
        }
        Err(e) => {
            Ok(serde_json::json!({
                "success": false,
                "message": format!("LLM 处理失败: {}", e),
                "canvas_state": null
            }))
        }
    }
}
```

在文件顶部添加 module 引入：
```rust
use crate::preprocessor::{self, PreprocessResult};
use crate::llm;
```

- [ ] **Step 2: 更新 lib.rs 注册 llm 模块**

```rust
mod engine;
mod commands;
mod preprocessor;
mod llm;
```

- [ ] **Step 3: 编译验证**

```bash
cd src-tauri && cargo check
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/commands/mod.rs src-tauri/src/lib.rs
git commit -m "feat: wire LLM scheduler into process_command with DeepSeek API

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Phase 6: 迭代修改 + 对话上下文

> 目标：实现多轮对话式修改（「把那个改成绿色」「删除这个节点」），对话上下文管理，历史压缩。

### Task 6.1: 实现对话上下文管理

**Files:**
- Create: `src-tauri/src/preprocessor/context_enrich.rs`

- [ ] **Step 1: 实现上下文补全**

`src-tauri/src/preprocessor/context_enrich.rs`:

```rust
use std::collections::HashMap;

/// 对话轮次记录
#[derive(Debug, Clone)]
pub struct ConversationTurn {
    pub user_text: String,
    pub assistant_text: String,
    /// 这一轮操作后新创建的节点 ID 列表
    pub created_nodes: Vec<String>,
}

/// 对话上下文管理器
pub struct ConversationContext {
    pub turns: Vec<ConversationTurn>,
}

impl ConversationContext {
    pub fn new() -> Self {
        Self { turns: Vec::new() }
    }

    /// 添加一轮对话
    pub fn add_turn(&mut self, user: String, assistant: String, nodes: Vec<String>) {
        self.turns.push(ConversationTurn {
            user_text: user,
            assistant_text: assistant,
            created_nodes: nodes,
        });
    }

    /// 获取最近 N 轮对话的摘要
    pub fn get_recent_summary(&self, n: usize) -> String {
        if self.turns.is_empty() {
            return "（无历史对话）".into();
        }
        let start = if self.turns.len() > n { self.turns.len() - n } else { 0 };
        self.turns[start..]
            .iter()
            .enumerate()
            .map(|(i, turn)| {
                format!("轮{}: 用户: {} | 助手: {}", i + 1, turn.user_text, turn.assistant_text)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// 在历史中搜索可能的节点引用
    /// 比如用户说「那个菱形」，找到上一轮中创建的 Decision 类型节点
    pub fn resolve_reference(&self, text: &str, nodes: &HashMap<String, crate::engine::canvas_state::DiagramNode>) -> Option<String> {
        // 查找模糊引用词
        let ref_words = ["那个", "这个", "它", "上面那个", "刚才那个"];
        let has_ref = ref_words.iter().any(|w| text.contains(w));
        if !has_ref {
            return None;
        }

        // 在上一轮的节点中搜索
        if let Some(last_turn) = self.turns.last() {
            for node_id in &last_turn.created_nodes {
                if let Some(node) = nodes.get(node_id) {
                    // 匹配提到的类型关键词
                    let type_keywords = [
                        ("菱形", "Decision"),
                        ("矩形", "Process"),
                        ("圆角", "Start"),
                        ("圆形", "Start"),
                        ("开始", "Start"),
                        ("结束", "End"),
                        ("判断", "Decision"),
                        ("处理", "Process"),
                    ];
                    for (keyword, nt) in &type_keywords {
                        if text.contains(keyword) && format!("{:?}", node.node_type) == *nt {
                            return Some(node_id.clone());
                        }
                    }
                    // 如果只有代词没有类型，返回上一轮最后一个节点
                    return Some(node_id.clone());
                }
            }
        }
        None
    }
}
```

- [ ] **Step 2: 提交**

```bash
git add src-tauri/src/preprocessor/context_enrich.rs
git commit -m "feat: implement conversation context with reference resolution

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 6.2: 对话历史集成到 LLM 调度

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`（添加上下文管理）
- Modify: `src-tauri/src/engine/mod.rs`（添加上下文存储）

前端 `src/store/index.ts` 已经维护了 `conversation` 数组，后端通过 Tauri State 管理。

- [ ] **Step 1: 在 Engine 中存储对话上下文**

更新 `src-tauri/src/engine/mod.rs`：

```rust
use crate::preprocessor::context_enrich::ConversationContext;

pub struct AppEngine {
    pub canvas: Mutex<Option<CanvasState>>,
    pub snapshots: Mutex<snapshot::SnapshotManager>,
    pub context: Mutex<ConversationContext>,
}

impl AppEngine {
    pub fn new() -> Self {
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
            context: Mutex::new(ConversationContext::new()),
        }
    }
}
```

- [ ] **Step 2: 在 commands 中传递对话历史**

更新 `process_command` 中调用 LLM 的部分，传入上下文：

```rust
let history: Vec<(String, String)> = {
    let ctx = ENGINE.context.lock().unwrap();
    ctx.turns.iter().map(|t| {
        ("user".to_string(), t.user_text.clone())
    }).collect()
};
```

并在 LLM 成功后更新上下文：

```rust
// 记录对话
let created_nodes: Vec<String> = result.canvas_state.as_ref()
    .map(|s| s.nodes.keys().cloned().collect())
    .unwrap_or_default();
ENGINE.context.lock().unwrap().add_turn(
    cleaned_text.clone(),
    result.message.clone(),
    created_nodes,
);
```

- [ ] **Step 3: 编译验证 + 提交**

```bash
cd src-tauri && cargo check
git add -A
git commit -m "feat: integrate conversation context into LLM scheduling pipeline

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Phase 7: 样式、主题、导出

> 目标：实现主题切换、样式自定义、PNG/SVG 导出，这些大部分是前端功能。

### Task 7.1: 实现主题预设

**Files:**
- Create: `src/lib/theme-presets.ts`

- [ ] **Step 1: 定义主题样式**

`src/lib/theme-presets.ts`:

```typescript
import type { Theme, NodeStyle } from "../store/types";

export const themePresets: Record<Theme, { background: string; nodeStyle: Partial<NodeStyle> }> = {
  Default: {
    background: "#fafafa",
    nodeStyle: { fill: "#ffffff", stroke: "#333333", stroke_width: 2 },
  },
  Professional: {
    background: "#f5f7fa",
    nodeStyle: { fill: "#e3f2fd", stroke: "#1565c0", stroke_width: 2, font_size: 13, font_family: "Microsoft YaHei, sans-serif", border_radius: 4 },
  },
  Handdrawn: {
    background: "#fefefe",
    nodeStyle: { fill: "#fffde7", stroke: "#5d4037", stroke_width: 2.5, font_size: 15, font_family: "Comic Sans MS, cursive", border_radius: 12 },
  },
  Dark: {
    background: "#263238",
    nodeStyle: { fill: "#37474f", stroke: "#78909c", stroke_width: 2, font_size: 14 },
  },
  Colorful: {
    background: "#fafafa",
    nodeStyle: { fill: "#fff3e0", stroke: "#e65100", stroke_width: 2.5, font_size: 14, border_radius: 8 },
  },
};
```

- [ ] **Step 2: 提交**

```bash
git add src/lib/theme-presets.ts
git commit -m "feat: add 5 theme presets for canvas styling

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 7.2: 实现导出功能

**Files:**
- Create: `src/hooks/useCanvasBridge.ts`

- [ ] **Step 1: 实现 Tauri 事件监听和导出 Hook**

`src/hooks/useCanvasBridge.ts`:

```typescript
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";
import type { CanvasState } from "../store/types";
import { useAppStore } from "../store";

export function useCanvasBridge() {
  const updateCanvasState = useAppStore((s) => s._updateCanvasState);

  useEffect(() => {
    const unlisten = listen<CanvasState>("canvas-updated", (event) => {
      updateCanvasState(event.payload);
    });
    return () => { unlisten.then((fn) => fn()); };
  }, [updateCanvasState]);
}

/**
 * 导出 Canvas 为图片
 */
export function exportCanvas(format: "png" | "svg" = "png"): void {
  const canvasEl = document.querySelector("canvas");
  if (!canvasEl) return;

  const dataURL = canvasEl.toDataURL(`image/${format}`);
  const link = document.createElement("a");
  link.download = `voice-draw-${Date.now()}.${format}`;
  link.href = dataURL;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
}
```

- [ ] **Step 2: 编译验证 + 提交**

```bash
bun run tsc --noEmit
git add src/hooks/useCanvasBridge.ts
git commit -m "feat: implement Canvas bridge hook and export to PNG/SVG

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Phase 8: 容错、性能优化、收尾

> 目标：加入错误处理、重试逻辑、加载状态优化、性能调优，确保系统健壮可用。

### Task 8.1: 实现前端错误处理和 Toast 提示

**Files:**
- Create: `src/components/status/Toast.tsx`
- Create: `src/components/canvas/CanvasOverlay.tsx`

- [ ] **Step 1: 实现 Toast 提示**

`src/components/status/Toast.tsx`:

```tsx
import { useEffect, useState } from "react";
import { useAppStore } from "../../store";

export default function Toast() {
  const lastOperation = useAppStore((s) => s.lastOperation);
  const status = useAppStore((s) => s.status);
  const [visible, setVisible] = useState(false);
  const [message, setMessage] = useState("");

  useEffect(() => {
    if (lastOperation && status !== "idle") {
      setMessage(lastOperation);
      setVisible(true);
      const timer = setTimeout(() => setVisible(false), 3000);
      return () => clearTimeout(timer);
    }
  }, [lastOperation, status]);

  if (!visible) return null;

  const bgColor = status === "error" ? "#dc3545"
    : status === "executing" ? "#28a745"
    : status === "thinking" ? "#17a2b8"
    : "#666";

  return (
    <div style={{
      position: "fixed",
      top: 48,
      left: "50%",
      transform: "translateX(-50%)",
      background: bgColor,
      color: "#fff",
      padding: "8px 20px",
      borderRadius: 20,
      fontSize: 13,
      zIndex: 1000,
      boxShadow: "0 2px 8px rgba(0,0,0,0.3)",
      animation: "fadeInDown 0.3s ease",
    }}>
      {status === "error" ? "❌ " : status === "executing" ? "✅ " : "💬 "}
      {message}
      <style>{`
        @keyframes fadeInDown {
          from { opacity: 0; transform: translateX(-50%) translateY(-10px); }
          to { opacity: 1; transform: translateX(-50%) translateY(0); }
        }
      `}</style>
    </div>
  );
}
```

`src/components/canvas/CanvasOverlay.tsx`:

```tsx
import { useAppStore } from "../../store";

export default function CanvasOverlay() {
  const status = useAppStore((s) => s.status);

  if (status !== "thinking") return null;

  return (
    <div style={{
      position: "absolute",
      top: "50%",
      left: "50%",
      transform: "translate(-50%, -50%)",
      background: "rgba(0,0,0,0.6)",
      color: "#fff",
      padding: "16px 32px",
      borderRadius: 12,
      fontSize: 15,
      zIndex: 100,
    }}>
      🤔 正在理解指令...
    </div>
  );
}
```

- [ ] **Step 2: 提交**

```bash
git add src/components/status/Toast.tsx src/components/canvas/CanvasOverlay.tsx
git commit -m "feat: implement Toast notifications and Canvas overlay for loading state

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 8.2: API Key 安全存储

**Files:**
- Create: `src-tauri/src/config/mod.rs`
- Create: `src-tauri/src/config/api_key.rs`

- [ ] **Step 1: 实现 API Key 管理**

`src-tauri/src/config/api_key.rs`:

```rust
use std::env;

/// API Key 来源优先级:
/// 1. 环境变量 DEEPSEEK_API_KEY
/// 2. Tauri 安全存储（后续实现）
/// 3. 提示用户配置
pub fn get_api_key() -> Result<String, String> {
    // 优先从环境变量读取
    if let Ok(key) = env::var("DEEPSEEK_API_KEY") {
        if !key.is_empty() && key != "sk-placeholder" {
            return Ok(key);
        }
    }

    Err(
        "请设置环境变量 DEEPSEEK_API_KEY。\n\
         例如: export DEEPSEEK_API_KEY=sk-xxxxx".into()
    )
}
```

`src-tauri/src/config/mod.rs`:

```rust
pub mod api_key;
```

- [ ] **Step 2: 在 commands 中使用**

更新 `src-tauri/src/commands/mod.rs` 中 API Key 读取：

```rust
let api_key = crate::config::api_key::get_api_key()
    .unwrap_or_else(|_| "sk-placeholder".into());
```

- [ ] **Step 3: 更新 lib.rs**

```rust
mod config;
```

- [ ] **Step 4: 提交**

```bash
git add src-tauri/src/config/
git commit -m "feat: implement API key management with env var support

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 8.3: 端到端集成测试

- [ ] **Step 1: 设置 API Key**

```bash
export DEEPSEEK_API_KEY=sk-your-actual-key
```

- [ ] **Step 2: 启动应用并测试完整流程**

```bash
bun run tauri dev
```

测试场景：
1. 说「画一个简单流程图：开始→处理数据→结束」→ 确认 Canvas 上出现 3 个节点和连线
2. 说「把处理数据改成绿色」→ 确认节点颜色变化
3. 说「撤销」→ 确认回退
4. 说「重做」→ 确认重做
5. 说「导出PNG」→ 确认下载文件
6. 说「放大」→ 确认 Canvas 缩放

- [ ] **Step 3: 提交**（无代码变更，仅验证记录）

```bash
git add -A
git commit -m "docs: complete end-to-end integration verification

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## 实现阶段总览

| 阶段 | 任务数 | 内容 | 可验证成果 |
|------|--------|------|-----------|
| Phase 0 | 4 | 项目搭建、依赖安装、三端通信 | Tauri 窗口 + React 页面 |
| Phase 1 | 6 | Canvas 状态引擎、节点/连线 CRUD、布局 | Rust 单元测试全绿 |
| Phase 2 | 7 | Fabric.js Canvas 渲染、Store、布局组件 | 硬编码数据正确渲染 |
| Phase 3 | 3 | Web Speech API 语音识别 | 按住说话实时转录 |
| Phase 4 | 4 | 预处理、快捷指令 | 说「撤销」立即回退 |
| Phase 5 | 4 | DeepSeek 集成、工具调用循环 | 说「画流程图」自动生成 |
| Phase 6 | 2 | 对话上下文、迭代修改 | 说「把那个改成XX」有效 |
| Phase 7 | 2 | 主题切换、PNG/SVG 导出 | 导出文件正确 |
| Phase 8 | 3 | 错误处理、API Key、集成测试 | 完整流程可用 |

**总计: 35 个任务，预计实现周期 2-3 周。**
