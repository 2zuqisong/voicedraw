# Grid Canvas Design Doc

> **Status:** Approved | **Date:** 2026-06-13

## Problem

When issuing two unrelated drawing commands, diagrams overlap because `auto_layout` always positions nodes relative to the same canvas origin. There is no concept of spatial zones or diagram groups.

## Solution

**Approach A: Absolute Grid Coordinates + Auto-Avoidance (混合模式)**

Convert the canvas to a coordinate grid (graph paper style). Each unit = 20px. Users can optionally specify `(grid_x, grid_y)` in voice commands, or omit them for automatic placement in the next available empty area.

### Grid Spec

| Property | Value |
|----------|-------|
| Grid unit size | 20px |
| Major line interval | Every 5 units (100px) |
| Minor line color | `#f0cccc` (淡红细线) |
| Major line color | `#e8b0b0` (稍深红线) |
| Origin offset | x=40px, y=24px (room for axis labels) |
| Default canvas | 1200 × 800, i.e. 58 × 38 grid units usable |

### Voice Command Examples

| User says | LLM interpretation |
|-----------|-------------------|
| "在坐标 (3, 2) 处画一个矩形" | `add_node(grid_x=3, grid_y=2, ...)` |
| "从 (18, 0) 到 (23, 6) 画椭圆" | `add_node(grid_x=18, grid_y=0, width=5, height=6)` |
| "画一个流程图" (no coords) | `add_nodes_batch(...)` — system auto-finds empty anchor |
| "在矩形右边空两格画圆" | LLM resolves relative position from context |

---

## Section 1: Data Model

### CanvasState additions

```rust
pub struct CanvasState {
    // ... existing fields ...
    pub grid_size: f64,        // 20.0
    pub grid_origin_x: f64,    // 40.0
    pub grid_origin_y: f64,    // 24.0
}
```

### New file: `src-tauri/src/engine/grid.rs`

```rust
pub struct GridConfig { grid_size, origin_x, origin_y }

impl GridConfig {
    pub fn default() -> Self
    pub fn grid_to_pixel(&self, gx: f64, gy: f64) -> (f64, f64)
    pub fn find_empty_anchor(&self, nodes: &HashMap<String, DiagramNode>) -> (f64, f64)
}
```

`find_empty_anchor` algorithm: scan left-to-right, top-to-bottom in 5-unit steps, check node bounding box overlap. Return first empty grid coordinate.

### Files touched

- `src-tauri/src/engine/canvas_state.rs` — add 3 grid fields
- `src-tauri/src/engine/grid.rs` — new
- `src-tauri/src/engine/mod.rs` — register `pub mod grid;`

---

## Section 2: LLM Tools & System Prompt

### Tool definition changes (`tool_defs.rs`)

- `add_node`: add optional `grid_x: number`, `grid_y: number`
- `add_nodes_batch`: add optional `grid_x: number`, `grid_y: number`
- `add_edge` / `add_edges_batch`: no change (edges reference node IDs)
- **New tool `get_empty_anchor`**: returns `{ grid_x, grid_y, pixel_x, pixel_y }` — the recommended empty anchor point

### System prompt addition (Rule 8)

```
8. **网格坐标系统**：
   - 画布为坐标网格，1 格 = 20 像素
   - 用户说"在 (x, y) 处..."时，将 grid_x/grid_y 填入工具参数
   - 用户未指定位置时，省略 grid_x/grid_y，系统自动找空白位置
   - 多个不相关的图应该放在不同区域，避免重叠
   - 画一个完整流程图（含多个节点）时，只需给第一个节点指定坐标或省略
```

### Scheduler changes (`scheduler.rs`)

- `execute_tool_call`: parse `grid_x`/`grid_y` from args, convert to pixel position, pass to node creation
- Add `"get_empty_anchor"` case to match arm

### Files touched

- `src-tauri/src/llm/tool_defs.rs`
- `src-tauri/src/llm/system_prompt.rs`
- `src-tauri/src/llm/scheduler.rs`

---

## Section 3: Frontend Rendering & Engine

### Frontend: grid background (`CanvasView.tsx`)

New `renderGrid()` function called before nodes/edges:
- Draw minor lines every `grid_size` px, color `#f0cccc`, width 0.3
- Draw major lines every `grid_size * 5` px, color `#e8b0b0`, width 0.8
- Lines are non-selectable, non-interactive (`selectable: false, evented: false`)
- Sent to back of canvas (`sendObjectToBack`)

### Backend: grid engine (`grid.rs`)

```rust
pub fn grid_to_pixel(gx, gy) -> (f64, f64)  // grid → pixel conversion
pub fn pixel_to_grid(px, py) -> (f64, f64)  // pixel → grid (for reverse lookup)
pub fn find_empty_anchor(nodes) -> (f64, f64)  // auto-placement scan
```

### Files touched

- `src/components/canvas/CanvasView.tsx` — add grid rendering
- `src-tauri/src/engine/grid.rs` — new
- `src-tauri/src/engine/mod.rs` — `pub mod grid;`

---

## Implementation Order

| Task | Description | Dependencies |
|------|-------------|-------------|
| 1 | Add grid fields to CanvasState + defaults | None |
| 2 | Implement `engine/grid.rs` (coord conversion + find_empty_anchor) | Task 1 |
| 3 | Update tool_defs + system_prompt (grid_x/grid_y + get_empty_anchor) | None |
| 4 | Update scheduler execute_tool_call (grid params + new tool) | Tasks 2, 3 |
| 5 | Frontend grid rendering in CanvasView.tsx | Task 1 |
| 6 | End-to-end test | All |
