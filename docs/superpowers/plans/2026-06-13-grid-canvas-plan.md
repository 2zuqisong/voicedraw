# Grid Canvas Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Convert the canvas to a coordinate grid (20px/unit, light red lines) with optional grid_x/grid_y parameters on tools, auto-placement for unspecified positions, preventing unrelated diagrams from overlapping.

**Architecture:** Add 3 grid fields to CanvasState. Create engine/grid.rs for coordinate conversion and empty-anchor detection. Update LLM tool definitions and system prompt to teach the model grid positioning. Update scheduler to parse grid params and dispatch a new get_empty_anchor tool. Render grid lines on the Fabric.js canvas in the frontend.

**Tech Stack:** Rust (engine), TypeScript + Fabric.js (frontend), async-openai (LLM tools)

---

## File Map

| Action | File | Responsibility |
|--------|------|----------------|
| Modify | `src-tauri/src/engine/canvas_state.rs:157` | Add grid_size, grid_origin_x, grid_origin_y fields |
| Modify | `src-tauri/src/engine/mod.rs:1,22-38` | Register grid module, init defaults |
| Create | `src-tauri/src/engine/grid.rs` | GridConfig struct, coord conversion, find_empty_anchor |
| Modify | `src-tauri/src/llm/tool_defs.rs:12-33,35-55,125-129` | Add grid_x/grid_y to tools, add get_empty_anchor |
| Modify | `src-tauri/src/llm/system_prompt.rs:21` | Add Rule 8: grid coordinate system |
| Modify | `src-tauri/src/llm/scheduler.rs:200-210,347-349` | Parse grid params, dispatch new tool |
| Modify | `src/components/canvas/CanvasView.tsx:36-51` | renderGrid function + call in renderCanvasState |

---

### Task 1: Add grid fields to CanvasState

**Files:**
- Modify: `src-tauri/src/engine/canvas_state.rs:147-157`
- Modify: `src-tauri/src/engine/mod.rs:22-38`

- [ ] **Step 1: Add grid_size, grid_origin_x, grid_origin_y to CanvasState**

Open `src-tauri/src/engine/canvas_state.rs` and add three fields after `height`:

```rust
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
    pub grid_size: f64,
    pub grid_origin_x: f64,
    pub grid_origin_y: f64,
}
```

- [ ] **Step 2: Update AppEngine::new() default CanvasState**

Open `src-tauri/src/engine/mod.rs`. Add grid field defaults to the `default_canvas`:

```rust
let default_canvas = CanvasState {
    id: "default".into(),
    title: "未命名图表".into(),
    nodes: HashMap::new(),
    edges: HashMap::new(),
    theme: Theme::Default,
    width: 1200.0,
    height: 800.0,
    grid_size: 20.0,
    grid_origin_x: 40.0,
    grid_origin_y: 24.0,
};
```

- [ ] **Step 3: Fix test fixtures that construct CanvasState directly**

All existing test fixtures in `engine/snapshot.rs` create `CanvasState` directly and will break with new fields. Update them:

In `src-tauri/src/engine/snapshot.rs`, find the `make_state` helper (around line 63) and add grid fields:

```rust
fn make_state(title: &str) -> CanvasState {
    CanvasState {
        id: "test".into(),
        title: title.into(),
        nodes: HashMap::new(),
        edges: HashMap::new(),
        theme: Theme::Default,
        width: 800.0,
        height: 600.0,
        grid_size: 20.0,
        grid_origin_x: 40.0,
        grid_origin_y: 24.0,
    }
}
```

- [ ] **Step 4: Build and run tests**

Run: `cd src-tauri && cargo test 2>&1`
Expected: 12 passed, 0 failed

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/engine/canvas_state.rs src-tauri/src/engine/mod.rs src-tauri/src/engine/snapshot.rs
git commit -m "feat: add grid_size, grid_origin_x, grid_origin_y to CanvasState

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2: Implement engine/grid.rs — coordinate conversion and empty-anchor detection

**Files:**
- Create: `src-tauri/src/engine/grid.rs`
- Modify: `src-tauri/src/engine/mod.rs:1-6` (add `pub mod grid;`)

- [ ] **Step 1: Write tests**

Create `src-tauri/src/engine/grid.rs` with tests first:

```rust
use super::canvas_state::*;
use std::collections::HashMap;

pub struct GridConfig {
    pub grid_size: f64,
    pub origin_x: f64,
    pub origin_y: f64,
}

impl GridConfig {
    pub fn default() -> Self {
        Self {
            grid_size: 20.0,
            origin_x: 40.0,
            origin_y: 24.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_to_pixel_origin() {
        let cfg = GridConfig::default();
        let (px, py) = cfg.grid_to_pixel(0.0, 0.0);
        assert_eq!(px, 40.0);
        assert_eq!(py, 24.0);
    }

    #[test]
    fn test_grid_to_pixel_positive() {
        let cfg = GridConfig::default();
        let (px, py) = cfg.grid_to_pixel(3.0, 2.0);
        assert_eq!(px, 100.0); // 40 + 3*20
        assert_eq!(py, 64.0);  // 24 + 2*20
    }

    #[test]
    fn test_pixel_to_grid() {
        let cfg = GridConfig::default();
        let (gx, gy) = cfg.pixel_to_grid(100.0, 64.0);
        assert_eq!(gx, 3.0);
        assert_eq!(gy, 2.0);
    }

    #[test]
    fn test_find_empty_anchor_when_canvas_empty() {
        let cfg = GridConfig::default();
        let nodes = HashMap::new();
        let (gx, gy) = cfg.find_empty_anchor(&nodes);
        assert_eq!(gx, 0.0);
        assert_eq!(gy, 0.0);
    }

    #[test]
    fn test_find_empty_anchor_avoids_occupied() {
        let cfg = GridConfig::default();
        let mut nodes = HashMap::new();
        // Place a node at grid (0,0), spanning ~8x3 grid units
        nodes.insert(
            "n1".into(),
            DiagramNode {
                id: "n1".into(),
                node_type: NodeType::Process,
                label: "占位".into(),
                position: Position { x: 40.0, y: 24.0 },
                size: Size { width: 160.0, height: 60.0 },
                style: NodeStyle::default(),
            },
        );
        let (gx, gy) = cfg.find_empty_anchor(&nodes);
        // First row is occupied at (0,0), so next empty should be further right or below
        // 160px width = 8 grid units, so column 9 or row below 3
        assert!(gx >= 8.0 || gy >= 3.0,
            "Expected empty anchor away from occupied node, got ({}, {})", gx, gy);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test grid 2>&1`
Expected: FAIL — method `grid_to_pixel` not found

- [ ] **Step 3: Implement GridConfig methods**

Add these methods inside `impl GridConfig` (before the `#[cfg(test)]` block):

```rust
impl GridConfig {
    pub fn default() -> Self {
        Self {
            grid_size: 20.0,
            origin_x: 40.0,
            origin_y: 24.0,
        }
    }

    /// 网格坐标 → 像素坐标
    pub fn grid_to_pixel(&self, gx: f64, gy: f64) -> (f64, f64) {
        (self.origin_x + gx * self.grid_size, self.origin_y + gy * self.grid_size)
    }

    /// 像素坐标 → 网格坐标
    pub fn pixel_to_grid(&self, px: f64, py: f64) -> (f64, f64) {
        (
            (px - self.origin_x) / self.grid_size,
            (py - self.origin_y) / self.grid_size,
        )
    }

    /// 扫描画布，返回第一个空白网格锚点
    /// 以 5 格为步长从左到右、从上到下扫描
    pub fn find_empty_anchor(
        &self,
        nodes: &HashMap<String, DiagramNode>,
    ) -> (f64, f64) {
        let step = 5.0; // 扫描步长
        let max_cols = ((1200.0 - self.origin_x) / (step * self.grid_size)).ceil() as i32;
        let max_rows = ((800.0 - self.origin_y) / (step * self.grid_size)).ceil() as i32;

        for row in 0..max_rows {
            for col in 0..max_cols {
                let gx = col as f64 * step;
                let gy = row as f64 * step;
                let (px, py) = self.grid_to_pixel(gx, gy);

                // 检查是否有节点与此区域重叠
                let occupied = nodes.values().any(|node| {
                    let nx = node.position.x;
                    let ny = node.position.y;
                    let nw = node.size.width;
                    let nh = node.size.height;
                    // 允许 40px 间距
                    let margin = 40.0;
                    px + margin < nx + nw
                        && px + 60.0 > nx - margin
                        && py + margin < ny + nh
                        && py + 60.0 > ny - margin
                });

                if !occupied {
                    return (gx, gy);
                }
            }
        }
        // 画布满了就返回原点
        (0.0, 0.0)
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test grid 2>&1`
Expected: 5 passed, 0 failed

- [ ] **Step 5: Register grid module in engine/mod.rs**

Open `src-tauri/src/engine/mod.rs`. Add `pub mod grid;` to the module list:

```rust
pub mod canvas_state;
pub mod node_ops;
pub mod edge_ops;
pub mod layout;
pub mod style_ops;
pub mod snapshot;
pub mod grid;
```

- [ ] **Step 6: Run full test suite**

Run: `cd src-tauri && cargo test 2>&1`
Expected: 17 passed, 0 failed (12 existing + 5 new)

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/engine/grid.rs src-tauri/src/engine/mod.rs
git commit -m "feat: implement grid coordinate conversion and empty-anchor detection

- GridConfig with grid_to_pixel and pixel_to_grid conversion
- find_empty_anchor scans canvas for unoccupied grid positions
- 5 tests covering conversion and collision avoidance

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 3: Update LLM tool definitions and system prompt

**Files:**
- Modify: `src-tauri/src/llm/tool_defs.rs:12-33,35-55`
- Modify: `src-tauri/src/llm/system_prompt.rs:20-21`

- [ ] **Step 1: Add grid_x/grid_y to add_nodes_batch tool definition**

Open `src-tauri/src/llm/tool_defs.rs`. In the `add_nodes_batch` definition (around line 12), add `grid_x` and `grid_y` as optional properties at the same level as `nodes`:

```rust
tool(
    "add_nodes_batch",
    "批量添加节点到画布",
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
            },
            "grid_x": {
                "type": "number",
                "description": "网格 X 坐标（可选，不填则自动找空位）。1格=20像素"
            },
            "grid_y": {
                "type": "number",
                "description": "网格 Y 坐标（可选，不填则自动找空位）。1格=20像素"
            }
        },
        "required": ["nodes"]
    }),
),
```

- [ ] **Step 2: Add grid_x/grid_y to add_node tool definition**

In the `add_node` definition (around line 57), add `grid_x` and `grid_y` inside the properties object:

```rust
tool(
    "add_node",
    "添加单个节点",
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
            },
            "grid_x": {
                "type": "number",
                "description": "网格 X 坐标（可选，不填则自动找空位）。1格=20像素"
            },
            "grid_y": {
                "type": "number",
                "description": "网格 Y 坐标（可选，不填则自动找空位）。1格=20像素"
            }
        },
        "required": ["type", "label"]
    }),
),
```

- [ ] **Step 3: Add get_empty_anchor tool**

Add a new tool entry after `get_canvas_state` (at the end of the vec, before the closing `]`):

```rust
tool(
    "get_empty_anchor",
    "获取画布上当前未被占用的推荐锚点坐标，用于放置新图表",
    json!({
        "type": "object",
        "properties": {},
        "required": []
    }),
),
```

- [ ] **Step 4: Update system prompt with Rule 8**

Open `src-tauri/src/llm/system_prompt.rs`. After rule 7 (before the closing `"#`), add:

```rust
7. **所有回复和标签使用中文**。
8. **网格坐标系统**：
   - 画布为坐标网格，1 格 = 20 像素，原点在左上角
   - 用户说"在 (x, y) 处..."时，将 grid_x/grid_y 填入工具参数（数字类型）
   - 用户未指定位置时，省略 grid_x/grid_y，系统自动找空白位置
   - 多个不相关的图（如先画矩形再画流程图），必须放在不同区域避免重叠
   - 如果不知道哪里有空位，先调用 get_empty_anchor 查询
   - 画完整流程图（含多个节点+连线）时，只需给第一个节点或批量接口指定锚点坐标，后续节点用 add_edge 关联即可
```

- [ ] **Step 5: Build to verify**

Run: `cd src-tauri && cargo build 2>&1`
Expected: 0 errors

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/llm/tool_defs.rs src-tauri/src/llm/system_prompt.rs
git commit -m "feat: add grid_x/grid_y params and get_empty_anchor tool to LLM definitions

- add_node and add_nodes_batch now accept optional grid coordinates
- New get_empty_anchor tool for LLM to query available canvas space
- System prompt Rule 8 teaches LLM grid coordinate system usage

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 4: Update scheduler to handle grid params and get_empty_anchor

**Files:**
- Modify: `src-tauri/src/llm/scheduler.rs:200-210,219-233,347-349`

- [ ] **Step 1: Update add_node case to parse grid_x/grid_y**

Open `src-tauri/src/llm/scheduler.rs`. In the `execute_tool_call` function, replace the `"add_node"` case:

```rust
"add_node" => {
    let node_type = crate::engine::canvas_state::NodeType::from_str(
        args["type"].as_str().unwrap_or("process"),
    )
    .map_err(|e| format!("{}", e))?;
    let label = args["label"].as_str().unwrap_or("未命名").to_string();

    // 如果提供了网格坐标，转换为像素位置
    let position = if let (Some(gx), Some(gy)) =
        (args["grid_x"].as_f64(), args["grid_y"].as_f64())
    {
        let grid_cfg = crate::engine::grid::GridConfig::default();
        let (px, py) = grid_cfg.grid_to_pixel(gx, gy);
        Some(crate::engine::canvas_state::Position { x: px, y: py })
    } else if let (Some(x), Some(y)) =
        (args["position"].as_object().and_then(|p| p["x"].as_f64()),
         args["position"].as_object().and_then(|p| p["y"].as_f64()))
    {
        Some(crate::engine::canvas_state::Position { x, y })
    } else {
        None
    };

    let node = crate::engine::node_ops::add_node(
        &mut state.nodes,
        node_type,
        label,
        position,
        None,
    );
    Ok(serde_json::json!({"node_id": node.id, "label": node.label}).to_string())
}
```

- [ ] **Step 2: Update add_nodes_batch case to parse grid_x/grid_y**

Replace the `"add_nodes_batch"` case:

```rust
"add_nodes_batch" => {
    let nodes = args["nodes"]
        .as_array()
        .ok_or("nodes 必须是数组")?;
    let batch: Vec<_> = nodes
        .iter()
        .map(|n| {
            let nt = crate::engine::canvas_state::NodeType::from_str(
                n["type"].as_str().unwrap_or("process"),
            )
            .unwrap_or(crate::engine::canvas_state::NodeType::Process);
            let label = n["label"].as_str().unwrap_or("未命名").to_string();
            (nt, label)
        })
        .collect();
    let created =
        crate::engine::node_ops::add_nodes_batch(&mut state.nodes, batch);

    // 如果提供了网格坐标，偏移所有新节点
    if let (Some(gx), Some(gy)) =
        (args["grid_x"].as_f64(), args["grid_y"].as_f64())
    {
        let grid_cfg = crate::engine::grid::GridConfig::default();
        let (base_px, base_py) = grid_cfg.grid_to_pixel(gx, gy);
        // 计算偏移量
        if let Some(first) = created.first() {
            let dx = base_px - first.position.x;
            let dy = base_py - first.position.y;
            for node in &created {
                if let Some(n) = state.nodes.get_mut(&node.id) {
                    n.position.x += dx;
                    n.position.y += dy;
                }
            }
        }
    } else if args["grid_x"].is_none() && args["grid_y"].is_none() {
        // 未指定坐标时，自动找空白位置
        let grid_cfg = crate::engine::grid::GridConfig::default();
        let (auto_gx, auto_gy) = grid_cfg.find_empty_anchor(&state.nodes);
        let (base_px, base_py) = grid_cfg.grid_to_pixel(auto_gx, auto_gy);
        if let Some(first) = created.first() {
            let dx = base_px - first.position.x;
            let dy = base_py - first.position.y;
            for node in &created {
                if let Some(n) = state.nodes.get_mut(&node.id) {
                    n.position.x += dx;
                    n.position.y += dy;
                }
            }
        }
    }

    // 自动布局
    let moved = crate::engine::layout::auto_layout(
        &mut state.nodes,
        &state.edges,
        crate::engine::layout::LayoutDirection::TopDown,
    );
    Ok(serde_json::json!({
        "added_count": created.len(),
        "layout_moved": moved,
        "nodes": created.iter().map(|n| {
            serde_json::json!({"id": n.id, "type": format!("{:?}", n.node_type), "label": n.label})
        }).collect::<Vec<_>>()
    })
    .to_string())
}
```

- [ ] **Step 3: Add get_empty_anchor case**

Add before the `_ => Err(format!("未知工具: {}", name)),` line:

```rust
"get_empty_anchor" => {
    let grid_cfg = crate::engine::grid::GridConfig::default();
    let (gx, gy) = grid_cfg.find_empty_anchor(&state.nodes);
    let (px, py) = grid_cfg.grid_to_pixel(gx, gy);
    Ok(serde_json::json!({
        "grid_x": gx,
        "grid_y": gy,
        "pixel_x": px,
        "pixel_y": py,
        "message": format!("推荐锚点: 网格({}, {}), 像素({}, {})", gx, gy, px, py)
    }).to_string())
}
```

- [ ] **Step 4: Build and run tests**

Run: `cd src-tauri && cargo build 2>&1 && cargo test 2>&1`
Expected: 0 errors, 17 passed

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/llm/scheduler.rs
git commit -m "feat: wire grid coordinates and auto-placement into LLM scheduler

- add_node parses optional grid_x/grid_y, converts to pixel position
- add_nodes_batch auto-finds empty anchor when no coordinates given
- get_empty_anchor tool returns recommended grid anchor point
- Unrelated diagrams now auto-place in different canvas regions

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 5: Frontend grid rendering in CanvasView

**Files:**
- Modify: `src/components/canvas/CanvasView.tsx:36-51`

- [ ] **Step 1: Add renderGrid function and call it**

Open `src/components/canvas/CanvasView.tsx`. Add `renderGrid` before `renderCanvasState`:

```tsx
/** 渲染坐标网格背景 */
function renderGrid(canvas: fabric.Canvas, state: CanvasState): void {
  const { grid_size, grid_origin_x, grid_origin_y, width, height } = state;

  // 细线（每格一条，淡红）
  for (let x = grid_origin_x; x <= width; x += grid_size) {
    const line = new fabric.Line([x, 0, x, height], {
      stroke: '#f0cccc',
      strokeWidth: 0.3,
      selectable: false,
      evented: false,
    });
    canvas.add(line);
    canvas.sendObjectToBack(line);
  }
  for (let y = grid_origin_y; y <= height; y += grid_size) {
    const line = new fabric.Line([0, y, width, y], {
      stroke: '#f0cccc',
      strokeWidth: 0.3,
      selectable: false,
      evented: false,
    });
    canvas.add(line);
    canvas.sendObjectToBack(line);
  }

  // 粗线（每 5 格一条，稍深）
  const majorStep = grid_size * 5;
  for (let x = grid_origin_x; x <= width; x += majorStep) {
    const line = new fabric.Line([x, 0, x, height], {
      stroke: '#e8b0b0',
      strokeWidth: 0.8,
      selectable: false,
      evented: false,
    });
    canvas.add(line);
    canvas.sendObjectToBack(line);
  }
  for (let y = grid_origin_y; y <= height; y += majorStep) {
    const line = new fabric.Line([0, y, width, y], {
      stroke: '#e8b0b0',
      strokeWidth: 0.8,
      selectable: false,
      evented: false,
    });
    canvas.add(line);
    canvas.sendObjectToBack(line);
  }
}
```

- [ ] **Step 2: Call renderGrid in renderCanvasState**

In the `renderCanvasState` function, add grid rendering as the first step, before clearing and re-rendering:

```tsx
function renderCanvasState(canvas: fabric.Canvas, state: CanvasState): void {
  canvas.clear();
  canvas.backgroundColor = state.theme === "Dark" ? "#263238" : "#fafafa";

  // 先渲染网格背景
  renderGrid(canvas, state);

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
```

- [ ] **Step 3: Check TypeScript store types include new grid fields**

Open `src/store/types.ts` and verify `CanvasState` interface has the new grid fields. If not, add them:

```tsx
export interface CanvasState {
  id: string;
  title: string;
  nodes: Record<string, DiagramNode>;
  edges: Record<string, DiagramEdge>;
  theme: string;
  width: number;
  height: number;
  grid_size: number;
  grid_origin_x: number;
  grid_origin_y: number;
}
```

- [ ] **Step 4: Build frontend to verify**

Run: `cd /home/zuqis/voice-to-draw && bun run build 2>&1 | tail -3`
Expected: `✓ built in X.XXs`

- [ ] **Step 5: Commit**

```bash
git add src/components/canvas/CanvasView.tsx src/store/types.ts
git commit -m "feat: render coordinate grid background on canvas

- renderGrid draws light red (#f0cccc) minor lines every 20px
- Major lines every 100px in slightly deeper red (#e8b0b0)
- Grid lines are non-interactive, sent to back of canvas
- CanvasState types updated with grid fields

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 6: End-to-end verification

**Files:** No code changes — verification only.

- [ ] **Step 1: Run all Rust tests**

Run: `cd src-tauri && cargo test 2>&1`
Expected: 17 passed, 0 failed

- [ ] **Step 2: Full app build**

Run: `cd /home/zuqis/voice-to-draw && bun run build 2>&1 | tail -3`
Expected: `✓ built in X.XXs`

- [ ] **Step 3: App startup test**

Run:
```bash
lsof -ti:1420 | xargs kill -9 2>/dev/null  # free port if needed
DEEPSEEK_API_KEY=sk-placeholder bun run tauri dev &
TAURI_PID=$!
sleep 12
if kill -0 $TAURI_PID 2>/dev/null; then
    echo "=== APP HEALTHY ==="
    kill $TAURI_PID 2>/dev/null
else
    echo "=== APP EXITED ==="
fi
```
Expected: `=== APP HEALTHY ===`

- [ ] **Step 4: Verify grid rendering (requires display)**

Launch app normally with a display attached. Confirm:
1. Canvas shows light red coordinate grid
2. Minor lines every 20px, major lines every 100px

- [ ] **Step 5: Verify LLM tool definitions (dry run)**

In a separate terminal, check that the LLM receives the new tool definitions:
```bash
# This is a manual check — start app with a real DEEPSEEK_API_KEY,
# send a command, and verify in logs that grid_x/grid_y params are available
RUST_LOG=debug DEEPSEEK_API_KEY=sk-real bun run tauri dev
```

- [ ] **Step 6: Commit verification record**

```bash
git add docs/superpowers/plans/2026-06-13-grid-canvas-plan.md
git commit -m "docs: complete grid canvas implementation and verification

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Self-Review

**1. Spec coverage:**
- Section 1 (Data Model): Task 1 (CanvasState fields) + Task 2 (GridConfig)
- Section 2 (LLM Tools): Task 3 (tool_defs + system_prompt) + Task 4 (scheduler)
- Section 3 (Frontend): Task 5 (CanvasView grid rendering)
- Implementation Order matches spec: 1→2→3→4→5→6

**2. Placeholder scan:** No TBD, TODO, or vague instructions. Every step has concrete code or exact commands.

**3. Type consistency:**
- `grid_size: f64` in CanvasState → `grid_size: number` in TypeScript interface → `grid_size` accessed in renderGrid ✓
- `GridConfig::grid_to_pixel(gx: f64, gy: f64) -> (f64, f64)` — called consistently in Task 2 tests and Task 4 scheduler ✓
- `find_empty_anchor(&HashMap<String, DiagramNode>) -> (f64, f64)` — signature consistent between grid.rs and scheduler calls ✓
- `grid_x`/`grid_y` as JSON number — parsed with `as_f64()` in scheduler ✓
