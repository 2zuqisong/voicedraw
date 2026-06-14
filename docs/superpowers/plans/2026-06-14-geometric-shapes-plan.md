# Geometric & Composite Shapes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the voice-draw system from flowchart-only to support basic geometric shapes (circle, rectangle, triangle, line, dot) and composite objects (house, sun, tree, smiley, star), with clear module separation — basic shapes shared, composites independent.

**Architecture:** Create a new `ShapeType` enum parallel to the existing `NodeType` — `NodeType` stays unchanged for flowchart types; `ShapeType` carries all geometric and composite variants. `DiagramNode` gets an optional `shape_type` field. The frontend's `renderNode` checks `shape_type` first (dispatch to `ShapeRenderer`), then falls through to the existing flowchart switch (zero regression risk). Composite shapes are stored as single `DiagramNode` entries with an embedded `sub_shapes` array; the frontend renders them as `fabric.Group`. A new `engine/shapes.rs` module provides the sub-shape recipes.

**Tech Stack:** Rust (Tauri 2.x backend), TypeScript/React 18 (frontend), Fabric.js v6 (canvas rendering), DeepSeek LLM (natural language → tool calls)

---

## File Structure Map

| File | Role |
|------|------|
| `src-tauri/src/engine/canvas_state.rs` | **NEW** `ShapeType` enum + `shape_type` field on `DiagramNode` + `SubShape` struct. `NodeType` **unchanged**. |
| `src-tauri/src/engine/shapes.rs` | **NEW** — composite shape recipes (functions returning `Vec<SubShape>`) |
| `src-tauri/src/engine/mod.rs` | Register new `shapes` module |
| `src-tauri/src/llm/tool_defs.rs` | Add `shape_type` parameter to `add_node` / `add_nodes_batch` (parallel to `type`) |
| `src-tauri/src/llm/prompts/system_prompt.md` | Add geometric shape instructions + few-shot examples using `shape_type` |
| `src-tauri/src/commands/mod.rs` | Handle `shape_type` arg in `execute_tool_call` — populate `sub_shapes` for composites |
| `src/store/types.ts` | Add `ShapeType` union + `shape_type` field + `SubShape` interface. `NodeType` **unchanged**. |
| `src/components/canvas/ShapeRenderer.ts` | **NEW** — pure functions: `renderBasicShape()`, `renderCompositeShape()` |
| `src/components/canvas/CanvasView.tsx` | `renderNode`: check `shape_type` first → ShapeRenderer, else fall through to existing flowchart switch |

---

### Task 1: Add ShapeType Enum, SubShape Struct, and Composite Shape Definitions

**Files:**
- Modify: `src-tauri/src/engine/canvas_state.rs`
- Create: `src-tauri/src/engine/shapes.rs`
- Modify: `src-tauri/src/engine/mod.rs`

- [ ] **Step 1: Add ShapeType enum and SubShape struct to canvas_state.rs**

In `src-tauri/src/engine/canvas_state.rs`, add `ShapeType` enum after the existing `NodeType` enum (keep `NodeType` completely untouched):

```rust
/// 几何图形类型（与流程图 NodeType 并列，互不干扰）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShapeType {
    // 基础几何图形
    Circle,
    Rectangle,
    Triangle,
    Line,
    Dot,
    // 复合图形
    House,
    Sun,
    Tree,
    Smiley,
    Star,
}

impl ShapeType {
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "circle" => Ok(Self::Circle),
            "rectangle" => Ok(Self::Rectangle),
            "triangle" => Ok(Self::Triangle),
            "line" => Ok(Self::Line),
            "dot" => Ok(Self::Dot),
            "house" => Ok(Self::House),
            "sun" => Ok(Self::Sun),
            "tree" => Ok(Self::Tree),
            "smiley" => Ok(Self::Smiley),
            "star" => Ok(Self::Star),
            _ => Err(format!("未知图形类型: {}", s)),
        }
    }
}
```

Add `SubShape` struct after `NodeStyle`:

```rust
/// 复合图形的子组件定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubShape {
    /// 子组件类型: "circle", "rect", "triangle", "line", "arc", "star_polygon"
    pub shape_type: String,
    /// 相对父节点左上角的 X 偏移
    pub rel_x: f64,
    /// 相对父节点左上角的 Y 偏移
    pub rel_y: f64,
    /// 宽度
    pub width: f64,
    /// 高度
    pub height: f64,
    /// 填充色
    pub fill: String,
    /// 边框色
    pub stroke: String,
    /// 边框宽度
    pub stroke_width: f64,
    /// 圆半径（圆形/点用）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<f64>,
}
```

Add `shape_type` and `sub_shapes` fields to `DiagramNode`:

```rust
pub struct DiagramNode {
    pub id: String,
    pub node_type: NodeType,
    /// 几何图形类型（与 node_type 并列，二选一生效；前端渲染时优先检查此字段）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape_type: Option<ShapeType>,
    pub label: String,
    pub position: Position,
    pub size: Size,
    pub style: NodeStyle,
    /// 复合图形的子组件列表（非复合图形为 None）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sub_shapes: Option<Vec<SubShape>>,
}
```

- [ ] **Step 2: Create composite shape definition module**

Create `src-tauri/src/engine/shapes.rs`:

```rust
use super::canvas_state::SubShape;

/// 获取复合图形类型的子组件定义
/// 返回 (Vec<SubShape>, 总宽度, 总高度)
pub fn get_composite_shapes(shape_type: &str) -> Option<(Vec<SubShape>, f64, f64)> {
    match shape_type {
        "house" => Some(house_shapes()),
        "sun" => Some(sun_shapes()),
        "tree" => Some(tree_shapes()),
        "smiley" => Some(smiley_shapes()),
        "star" => Some(star_shapes()),
        _ => None,
    }
}

/// 判断 shape_type 字符串是否为复合图形
pub fn is_composite(shape_type: &str) -> bool {
    matches!(shape_type, "house" | "sun" | "tree" | "smiley" | "star")
}

/// 判断 shape_type 字符串是否为基本几何图形
pub fn is_basic_shape(shape_type: &str) -> bool {
    matches!(shape_type, "circle" | "rectangle" | "triangle" | "line" | "dot")
}

fn sub(
    shape_type: &str,
    rel_x: f64, rel_y: f64,
    width: f64, height: f64,
    fill: &str, stroke: &str, stroke_width: f64,
    radius: Option<f64>,
) -> SubShape {
    SubShape {
        shape_type: shape_type.to_string(),
        rel_x, rel_y, width, height,
        fill: fill.to_string(),
        stroke: stroke.to_string(),
        stroke_width,
        radius,
    }
}

/// 房子 = 三角形屋顶 + 矩形屋身 + 矩形门 + 矩形窗
fn house_shapes() -> (Vec<SubShape>, f64, f64) {
    let shapes = vec![
        sub("triangle", 0.0, 0.0, 140.0, 60.0, "#ff9800", "#e65100", 2.0, None),
        sub("rect", 0.0, 60.0, 140.0, 90.0, "#e8f5e9", "#2e7d32", 2.0, None),
        sub("rect", 50.0, 100.0, 40.0, 50.0, "#795548", "#4e342e", 1.5, None),
        sub("rect", 10.0, 75.0, 30.0, 30.0, "#bbdefb", "#1565c0", 1.5, None),
    ];
    (shapes, 140.0, 150.0)
}

/// 太阳 = 圆形 + 8 条放射线
fn sun_shapes() -> (Vec<SubShape>, f64, f64) {
    let cx = 60.0;
    let cy = 60.0;
    let r = 40.0;
    let ray_len = 20.0;
    let mut shapes = vec![
        sub("circle", 20.0, 20.0, 80.0, 80.0, "#ffc107", "#f57f17", 2.0, Some(40.0)),
    ];
    for i in 0..8 {
        let angle = (i as f64) * std::f64::consts::PI / 4.0;
        let x1 = cx + r * angle.cos();
        let y1 = cy + r * angle.sin();
        let x2 = cx + (r + ray_len) * angle.cos();
        let y2 = cy + (r + ray_len) * angle.sin();
        let lx = x1.min(x2);
        let ly = y1.min(y2);
        let lw = (x2 - x1).abs().max(1.0);
        let lh = (y2 - y1).abs().max(1.0);
        shapes.push(sub("line", lx, ly, lw, lh, "#ffc107", "#f57f17", 3.0, None));
    }
    (shapes, 120.0, 120.0)
}

/// 树 = 绿色圆形树冠 + 棕色矩形树干
fn tree_shapes() -> (Vec<SubShape>, f64, f64) {
    let shapes = vec![
        sub("circle", 0.0, 0.0, 100.0, 90.0, "#4caf50", "#2e7d32", 2.0, Some(45.0)),
        sub("rect", 35.0, 80.0, 30.0, 70.0, "#795548", "#4e342e", 2.0, None),
    ];
    (shapes, 100.0, 150.0)
}

/// 笑脸 = 黄色圆脸 + 两个黑眼 + 弧形嘴巴
fn smiley_shapes() -> (Vec<SubShape>, f64, f64) {
    let shapes = vec![
        sub("circle", 0.0, 0.0, 100.0, 100.0, "#ffeb3b", "#f9a825", 2.0, Some(50.0)),
        sub("circle", 28.0, 30.0, 14.0, 14.0, "#333333", "#333333", 0.0, Some(7.0)),
        sub("circle", 58.0, 30.0, 14.0, 14.0, "#333333", "#333333", 0.0, Some(7.0)),
        sub("arc", 32.0, 55.0, 36.0, 20.0, "transparent", "#333333", 2.0, None),
    ];
    (shapes, 100.0, 100.0)
}

/// 五角星（前端用 star_polygon 渲染）
fn star_shapes() -> (Vec<SubShape>, f64, f64) {
    let shapes = vec![
        sub("star_polygon", 0.0, 0.0, 120.0, 120.0, "#ffd600", "#f57f17", 2.0, None),
    ];
    (shapes, 120.0, 120.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_house_has_four_sub_shapes() {
        let (shapes, w, h) = house_shapes();
        assert_eq!(shapes.len(), 4);
        assert!(w > 0.0 && h > 0.0);
    }

    #[test]
    fn test_sun_has_nine_sub_shapes() {
        let (shapes, w, h) = sun_shapes();
        assert_eq!(shapes.len(), 9);
    }

    #[test]
    fn test_tree_has_two_sub_shapes() {
        let (shapes, w, h) = tree_shapes();
        assert_eq!(shapes.len(), 2);
    }

    #[test]
    fn test_smiley_has_four_sub_shapes() {
        let (shapes, w, h) = smiley_shapes();
        assert_eq!(shapes.len(), 4);
    }

    #[test]
    fn test_star_has_one_polygon() {
        let (shapes, _w, _h) = star_shapes();
        assert_eq!(shapes.len(), 1);
        assert_eq!(shapes[0].shape_type, "star_polygon");
    }

    #[test]
    fn test_is_composite() {
        assert!(is_composite("house"));
        assert!(is_composite("star"));
        assert!(!is_composite("circle"));
        assert!(!is_composite("process"));
    }

    #[test]
    fn test_is_basic_shape() {
        assert!(is_basic_shape("circle"));
        assert!(is_basic_shape("triangle"));
        assert!(!is_basic_shape("house"));
        assert!(!is_basic_shape("process"));
    }

    #[test]
    fn test_unknown_returns_none() {
        assert!(get_composite_shapes("unknown").is_none());
        assert!(get_composite_shapes("process").is_none());
    }
}
```

- [ ] **Step 3: Register shapes module**

In `src-tauri/src/engine/mod.rs`, add `pub mod shapes;`:

```rust
pub mod canvas_state;
pub mod node_ops;
pub mod edge_ops;
pub mod layout;
pub mod style_ops;
pub mod snapshot;
pub mod grid;
pub mod shapes;  // <-- NEW

pub use canvas_state::*;
// ... rest unchanged
```

- [ ] **Step 4: Build Rust to verify compilation**

Run: `cd src-tauri && cargo build 2>&1`
Expected: Compiles successfully (shapes module unused warnings OK until later tasks).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/engine/canvas_state.rs src-tauri/src/engine/shapes.rs src-tauri/src/engine/mod.rs
git commit -m "feat(engine): add ShapeType enum and composite shape definitions

- Add ShapeType enum (Circle, Rectangle, Triangle, Line, Dot, House, Sun,
  Tree, Smiley, Star) parallel to existing NodeType — NodeType unchanged
- Add SubShape struct for composite shape sub-component definitions
- Add optional shape_type and sub_shapes fields to DiagramNode
- Create engine/shapes.rs with composite shape recipes + unit tests"
```

---

### Task 2: Extend LLM Tool Definitions with shape_type Parameter

**Files:**
- Modify: `src-tauri/src/llm/tool_defs.rs`

- [ ] **Step 1: Add shape_type parameter to add_node and add_nodes_batch tools**

In both `add_nodes_batch` and `add_node` tool definitions, add a `shape_type` property **alongside** the existing `type` property (not replacing it):

For `add_nodes_batch`, change the nodes items schema from:
```rust
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
```
to:
```rust
"properties": {
    "type": {
        "type": "string",
        "enum": ["start", "end", "process", "decision", "data", "subprocess", "text"],
        "description": "流程图节点类型（与 shape_type 二选一）"
    },
    "shape_type": {
        "type": "string",
        "enum": ["circle", "rectangle", "triangle", "line", "dot", "house", "sun", "tree", "smiley", "star"],
        "description": "几何图形类型（与 type 二选一，画几何图形时用此字段）"
    },
    "label": {
        "type": "string",
        "description": "节点显示的文本标签"
    }
},
```

Apply the **same change** to the `add_node` tool definition (both have the same nodes/type properties structure).

Note: The existing `type` field stays exactly as-is. `shape_type` is a new optional field. LLM will use `type` for flowcharts and `shape_type` for geometric shapes.

- [ ] **Step 2: Build Rust to verify compilation**

Run: `cd src-tauri && cargo build 2>&1`
Expected: Compiles successfully.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/llm/tool_defs.rs
git commit -m "feat(llm): add shape_type parameter to tool definitions

Add shape_type field (parallel to existing type) in add_node and
add_nodes_batch tool schemas. LLM uses 'type' for flowcharts and
'shape_type' for geometric/composite shapes — clean separation."
```

---

### Task 3: Update System Prompt for Geometric Shapes

**Files:**
- Modify: `src-tauri/src/llm/prompts/system_prompt.md`

- [ ] **Step 1: Rewrite "基础形状映射" section**

Replace the existing "## 基础形状映射" section (lines 19-26) with the expanded version below. The key change: all geometric shapes use `shape_type` (not `type`):

```markdown
## 基础几何图形映射
当用户说"画一个X"且 X 是几何图形（非流程图概念）时，使用 shape_type 参数，直接执行不要反问。

### 基本图形（用 shape_type）
- "画圆形"/"画圆"/"画一个红色圆形"/"画半径为50的蓝色圆" → shape_type="circle"
  - 颜色识别：支持常见色名（红/黄/蓝/绿/黑/白/橙/紫/灰/粉/棕）和十六进制
  - 半径识别："半径为N"→ 用 size 的 width/height 表达直径
- "画矩形"/"画方形"/"画一个黄色矩形，宽100高80"/"画绿色正方形" → shape_type="rectangle"
  - 宽高识别："宽N高M"→ size.width=N, size.height=M；"正方形"→ width=height
- "画三角形"/"画一个黑色三角形，边长120" → shape_type="triangle"
- "画线段"/"从坐标100,200画一条线到300,400" → shape_type="line"
- "画点"/"在位置50,60画一个红色点" → shape_type="dot"

### 复合图形（用 shape_type，单节点自动展开为多组件）
- "画房子"/"画一座蓝色房子" → shape_type="house"
- "画太阳"/"画一个金色太阳" → shape_type="sun"
- "画树"/"画一棵绿色的树" → shape_type="tree"
- "画笑脸"/"画一个笑脸" → shape_type="smiley"
- "画星星"/"画一个五角星" → shape_type="star"

### 流程图节点（用 type，保持不变）
- "画矩形"/"画方形" → 见上方基本图形，用 shape_type="rectangle"
- 如果是流程图语境（"画登录流程"），才用 type="start"/"process"/"decision" 等

重要区分：几何图形用 shape_type，流程图用 type。两者不混用。

## 坐标口语化支持
支持以下相对位置词，自动转换为网格坐标：
- "左上角"/"左上" → grid_x=2, grid_y=2
- "右上角"/"右上" → grid_x=40, grid_y=2
- "左下角"/"左下" → grid_x=2, grid_y=30
- "右下角"/"右下" → grid_x=40, grid_y=30
- "中心"/"中间"/"中央" → grid_x=20, grid_y=15
- "左边" → 从当前位置向左偏移约 8 格
- "右边" → 从当前位置向右偏移约 8 格
- "上边"/"上面" → 从当前位置向上偏移约 6 格
- "下边"/"下面" → 从当前位置向下偏移约 6 格
- "在X的左边"/"在X旁边" → 参考对话历史中 X 的位置计算

### 颜色速查表
| 口语 | 颜色值 |
|------|--------|
| 红/红色 | #e53935 |
| 蓝/蓝色 | #1e88e5 |
| 绿/绿色 | #43a047 |
| 黄/黄色 | #fdd835 |
| 橙/橙色 | #fb8c00 |
| 紫/紫色 | #8e24aa |
| 黑/黑色 | #212121 |
| 白/白色 | #fafafa |
| 灰/灰色 | #9e9e9e |
| 粉/粉色 | #ec407a |
| 棕/棕色 | #795548 |
| 金/金色 | #ffc107 |

原则：只要能从类型表中找到匹配的形状，就默认执行。用户后续可以修改。
```

- [ ] **Step 2: Add few-shot examples for geometric shapes**

Add before the final "## 提醒" section:

```markdown
### 示例 7：基本几何图形（用 shape_type）
用户: "画一个红色圆形，再画一个蓝色矩形"
助手思考: 两个独立几何图形，用 shape_type，无连线。
工具调用:
  [add_nodes_batch] nodes=[
    {"shape_type":"circle","label":"圆形"},
    {"shape_type":"rectangle","label":"矩形"}
  ] grid_x=2 grid_y=2
  [update_node] node_id="<circle_id>" fill="#e53935"
  [update_node] node_id="<rect_id>" fill="#1e88e5"
回复: "已画出红色圆形和蓝色矩形。"

### 示例 8：复合图形——房子（单节点自动展开）
用户: "画一座蓝色房子"
助手思考: 房子是复合图形，单个 shape_type="house" 节点即可，后端自动填充子组件。
工具调用:
  [add_node] shape_type="house" label="房子" fill="#1e88e5"
回复: "已画出一座蓝色房子，含屋顶、屋身、门和窗。"

### 示例 9：复合图形 + 相对位置
用户: "在房子左边画一棵树"
助手思考: 对话历史中有房子节点，它的左边约偏移8格。树用 shape_type="tree"。
工具调用:
  [add_node] shape_type="tree" label="树" grid_x=<house_grid_x - 8> grid_y=<house_grid_y>
回复: "已在房子左边画了一棵树。"

### 示例 10：属性微调
用户: "把房子改成红色"
助手思考: 复合图形的整体颜色可通过 update_node 修改 fill。
工具调用:
  [update_node] node_id="<house_id>" fill="#e53935"
回复: "已将房子改为红色。"

### 示例 11：多图形组合
用户: "画一个笑脸，在它右边画一个五角星"
助手思考: 两个复合图形，左右排列，用 shape_type="smiley" 和 shape_type="star"。
工具调用:
  [add_nodes_batch] nodes=[
    {"shape_type":"smiley","label":"笑脸"},
    {"shape_type":"star","label":"五角星"}
  ] grid_x=2 grid_y=10
  [auto_layout] direction="left_right"
回复: "已画出笑脸和五角星，左右排列。"
```

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/llm/prompts/system_prompt.md
git commit -m "feat(llm): add geometric shapes and composite objects to system prompt

- Rewrite shape mapping section: geometric shapes use shape_type,
  flowchart nodes use type — clean separation, never mixed
- Add coordinate colloquialism support with grid position table
- Add color reference table (12 common colors)
- Add 5 new few-shot examples (7-11) covering basic shapes,
  composite objects, relative positioning, and color adjustments"
```

---

### Task 4: Extend Frontend TypeScript Types

**Files:**
- Modify: `src/store/types.ts`

- [ ] **Step 1: Add ShapeType, SubShape, update DiagramNode**

In `src/store/types.ts`, add `ShapeType` after the existing `NodeType` (keep `NodeType` completely untouched):

```typescript
// NodeType stays exactly as-is — flowchart types only
export type NodeType =
  | "Start"
  | "End"
  | "Process"
  | "Decision"
  | "Data"
  | "Subprocess"
  | "Text";

// NEW: ShapeType — geometric and composite shapes
export type ShapeType =
  // Basic geometric shapes
  | "Circle"
  | "Rectangle"
  | "Triangle"
  | "Line"
  | "Dot"
  // Composite shapes
  | "House"
  | "Sun"
  | "Tree"
  | "Smiley"
  | "Star";
```

Add `SubShape` interface after `NodeStyle`:

```typescript
/** 复合图形的子组件定义 */
export interface SubShape {
  shape_type: string;
  rel_x: number;
  rel_y: number;
  width: number;
  height: number;
  fill: string;
  stroke: string;
  stroke_width: number;
  radius?: number;
}
```

Update `DiagramNode` — add `shape_type` and `sub_shapes`:

```typescript
export interface DiagramNode {
  id: string;
  node_type: NodeType;
  /** 几何图形类型（与 node_type 并列，二选一；渲染时优先检查此字段） */
  shape_type?: ShapeType;
  label: string;
  position: Position;
  size: Size;
  style: NodeStyle;
  /** 复合图形的子组件列表（非复合图形为 undefined） */
  sub_shapes?: SubShape[];
}
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `cd /home/zuqis/voice-to-draw && npx tsc --noEmit 2>&1`
Expected: No errors from types.ts.

- [ ] **Step 3: Commit**

```bash
git add src/store/types.ts
git commit -m "feat(frontend): add ShapeType, SubShape to TypeScript types

- Add ShapeType union (10 variants) parallel to existing NodeType
- Add SubShape interface for composite shape sub-components
- Add optional shape_type and sub_shapes fields to DiagramNode
- NodeType union completely unchanged — zero regression risk"
```

---

### Task 5: Create ShapeRenderer Module

**Files:**
- Create: `src/components/canvas/ShapeRenderer.ts`

- [ ] **Step 1: Write ShapeRenderer with basic and composite shape rendering**

Create `src/components/canvas/ShapeRenderer.ts`:

```typescript
import * as fabric from "fabric";
import type { DiagramNode, SubShape } from "../../store/types";

/**
 * 渲染基本几何图形，返回 fabric.Object
 */
export function renderBasicShape(
  node: DiagramNode,
): fabric.Object {
  const { position, size, style, shape_type } = node;
  const cx = position.x + size.width / 2;
  const cy = position.y + size.height / 2;

  switch (shape_type) {
    case "Circle": {
      const rx = size.width / 2;
      const ry = size.height / 2;
      return new fabric.Ellipse({
        left: cx,
        top: cy,
        rx,
        ry,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
        originX: "center",
        originY: "center",
      });
    }

    case "Rectangle": {
      return new fabric.Rect({
        left: position.x,
        top: position.y,
        width: size.width,
        height: size.height,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
        rx: style.border_radius,
        ry: style.border_radius,
      });
    }

    case "Triangle": {
      return new fabric.Triangle({
        left: cx,
        top: cy,
        width: size.width,
        height: size.height,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
        originX: "center",
        originY: "center",
      });
    }

    case "Line": {
      return new fabric.Line(
        [position.x, position.y, position.x + size.width, position.y + size.height],
        {
          stroke: style.stroke || "#333333",
          strokeWidth: style.stroke_width || 2,
        },
      );
    }

    case "Dot": {
      const r = style.stroke_width > 0 ? style.stroke_width * 3 : 8;
      return new fabric.Ellipse({
        left: position.x,
        top: position.y,
        rx: r,
        ry: r,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: 0.5,
        originX: "center",
        originY: "center",
      });
    }

    default:
      // Fallback
      return new fabric.Rect({
        left: position.x,
        top: position.y,
        width: size.width,
        height: size.height,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
      });
  }
}

/**
 * 渲染复合图形，返回 fabric.Group
 */
export function renderCompositeShape(
  node: DiagramNode,
): fabric.Group {
  const subShapes = node.sub_shapes;
  const objects: fabric.Object[] = [];

  if (!subShapes || subShapes.length === 0) {
    return new fabric.Group([
      new fabric.Rect({
        width: node.size.width,
        height: node.size.height,
        fill: node.style.fill,
        stroke: node.style.stroke,
        strokeWidth: node.style.stroke_width,
      }),
    ]);
  }

  for (const sub of subShapes) {
    const obj = renderSubShape(sub);
    if (obj) {
      obj.set({ left: sub.rel_x, top: sub.rel_y });
      objects.push(obj);
    }
  }

  return new fabric.Group(objects, {
    left: node.position.x,
    top: node.position.y,
  });
}

/** 渲染单个子组件 */
function renderSubShape(sub: SubShape): fabric.Object | null {
  switch (sub.shape_type) {
    case "rect": {
      return new fabric.Rect({
        width: sub.width,
        height: sub.height,
        fill: sub.fill,
        stroke: sub.stroke,
        strokeWidth: sub.stroke_width,
      });
    }

    case "circle": {
      const r = sub.radius ?? Math.min(sub.width, sub.height) / 2;
      return new fabric.Ellipse({
        left: r,
        top: r,
        rx: r,
        ry: r,
        fill: sub.fill,
        stroke: sub.stroke,
        strokeWidth: sub.stroke_width,
        originX: "center",
        originY: "center",
      });
    }

    case "triangle": {
      return new fabric.Triangle({
        width: sub.width,
        height: sub.height,
        fill: sub.fill,
        stroke: sub.stroke,
        strokeWidth: sub.stroke_width,
      });
    }

    case "line": {
      const cx = sub.width / 2;
      const cy = sub.height / 2;
      return new fabric.Line(
        [cx, cy - sub.height / 2, cx, cy + sub.height / 2],
        {
          stroke: sub.stroke,
          strokeWidth: sub.stroke_width,
        },
      );
    }

    case "arc": {
      return new fabric.Ellipse({
        left: sub.width / 2,
        top: 0,
        rx: sub.width / 2,
        ry: sub.height,
        fill: sub.fill,
        stroke: sub.stroke,
        strokeWidth: sub.stroke_width,
        originX: "center",
        originY: "top",
      });
    }

    case "star_polygon": {
      return createStarPolygon(sub);
    }

    default:
      return new fabric.Rect({
        width: sub.width,
        height: sub.height,
        fill: sub.fill,
        stroke: sub.stroke,
        strokeWidth: sub.stroke_width,
      });
  }
}

/** 五角星多边形 */
function createStarPolygon(sub: SubShape): fabric.Polygon {
  const cx = sub.width / 2;
  const cy = sub.height / 2;
  const outerR = sub.width / 2;
  const innerR = outerR * 0.4;
  const points = 5;
  const vertices: { x: number; y: number }[] = [];

  for (let i = 0; i < points * 2; i++) {
    const angle = (i * Math.PI) / points - Math.PI / 2;
    const r = i % 2 === 0 ? outerR : innerR;
    vertices.push({
      x: cx + r * Math.cos(angle),
      y: cy + r * Math.sin(angle),
    });
  }

  return new fabric.Polygon(vertices, {
    left: 0,
    top: 0,
    fill: sub.fill,
    stroke: sub.stroke,
    strokeWidth: sub.stroke_width,
  });
}

/** 判断 shape_type 是否为复合图形 */
export function isCompositeShape(shapeType: string): boolean {
  return ["House", "Sun", "Tree", "Smiley", "Star"].includes(shapeType);
}

/** 判断 shape_type 是否为基本几何图形 */
export function isBasicShape(shapeType: string): boolean {
  return ["Circle", "Rectangle", "Triangle", "Line", "Dot"].includes(shapeType);
}
```

- [ ] **Step 2: Verify TypeScript compilation**

Run: `cd /home/zuqis/voice-to-draw && npx tsc --noEmit 2>&1`
Expected: No errors in ShapeRenderer.ts.

- [ ] **Step 3: Commit**

```bash
git add src/components/canvas/ShapeRenderer.ts
git commit -m "feat(frontend): add ShapeRenderer module for geometric and composite shapes

- renderBasicShape(): renders Circle, Rectangle, Triangle, Line, Dot
- renderCompositeShape(): reads node.sub_shapes and creates fabric.Group
- renderSubShape(): dispatches sub-component types (rect, circle, triangle,
  line, arc, star_polygon)
- createStarPolygon(): 5-pointed star via alternating vertex radii
- Predicates check shape_type, not node_type"
```

---

### Task 6: Update CanvasView renderNode to Dispatch on shape_type

**Files:**
- Modify: `src/components/canvas/CanvasView.tsx`

- [ ] **Step 1: Import ShapeRenderer helpers**

Add import at top of CanvasView.tsx (after the existing imports):

```typescript
import {
  renderBasicShape,
  renderCompositeShape,
  isCompositeShape,
  isBasicShape,
} from "./ShapeRenderer";
```

- [ ] **Step 2: Add shape_type dispatch before existing flowchart switch**

In the `renderNode` function, add the shape_type check at the very beginning (after destructuring), before the existing flowchart switch. The existing switch block stays completely unchanged:

```typescript
/** 渲染单个节点 */
function renderNode(
  canvas: fabric.Canvas,
  node: CanvasState["nodes"][string],
): void {
  const { position, size, style, label, node_type, shape_type } = node;

  // --- NEW: Composite shapes (check shape_type first) ---
  if (shape_type && isCompositeShape(shape_type)) {
    const group = renderCompositeShape(node);
    (group as any).data = { nodeId: node.id, nodeType: node_type, shapeType: shape_type };

    if (label && label.length > 0) {
      const text = new fabric.Text(label, {
        left: 0,
        top: -22,
        fontSize: style.font_size,
        fontFamily: style.font_family,
        fill: "#1a1a1a",
        textAlign: "center",
      });
      text.set({ left: (node.size.width - text.width) / 2 });
      group.add(text);
    }

    canvas.add(group);
    return;
  }

  // --- NEW: Basic geometric shapes (check shape_type first) ---
  if (shape_type && isBasicShape(shape_type)) {
    const shape = renderBasicShape(node);
    const text = new fabric.Text(label, {
      left: position.x + size.width / 2,
      top: position.y + size.height / 2,
      fontSize: style.font_size,
      fontFamily: style.font_family,
      fill: "#1a1a1a",
      originX: "center",
      originY: "center",
      textAlign: "center",
    });

    const group = new fabric.Group([shape, text], {
      left: position.x,
      top: position.y,
    });
    (group as any).data = { nodeId: node.id, nodeType: node_type, shapeType: shape_type };
    canvas.add(group);
    return;
  }

  // --- Existing flowchart node rendering (COMPLETELY UNCHANGED below this line) ---
  let shape: fabric.Object;

  switch (node_type) {
    case "Start":
    case "End":
      // ... all existing code unchanged ...
  }
  // ... rest of existing function unchanged ...
}
```

Note: The existing flowchart switch block (everything from `let shape: fabric.Object;` onwards) stays **completely unchanged**. The new code is inserted before it and returns early when `shape_type` is set, so existing flowchart rendering is never affected.

- [ ] **Step 2: Verify TypeScript compilation**

Run: `cd /home/zuqis/voice-to-draw && npx tsc --noEmit 2>&1`
Expected: No errors.

- [ ] **Step 3: Commit**

```bash
git add src/components/canvas/CanvasView.tsx
git commit -m "feat(frontend): dispatch shape_type to ShapeRenderer in renderNode

- Check node.shape_type first: if set, dispatch to ShapeRenderer
  (basic shapes → renderBasicShape, composite → renderCompositeShape)
- Early return after shape rendering prevents fall-through
- Existing flowchart switch block completely untouched — zero regression
- Label rendered above composite shape groups"
```

---

### Task 7: Handle shape_type in Backend Command Execution

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: Update execute_tool_call to handle shape_type**

In the `execute_tool_call` function, update the `"add_node"` match arm. The key change: check for `shape_type` arg in addition to `type`:

```rust
"add_node" => {
    // Check for shape_type first (geometric shapes)
    let shape_type_str = args["shape_type"].as_str();
    let node_type_str = args["type"].as_str().unwrap_or("process");

    let label = args["label"].as_str().unwrap_or("未命名").to_string();

    let position = if let (Some(gx), Some(gy)) =
        (args["grid_x"].as_f64(), args["grid_y"].as_f64())
    {
        let grid_cfg = crate::engine::grid::GridConfig::default();
        let (px, py) = grid_cfg.grid_to_pixel(gx, gy);
        Some(crate::engine::canvas_state::Position { x: px, y: py })
    } else if let (Some(x), Some(y)) = (
        args["position"].as_object().and_then(|p| p["x"].as_f64()),
        args["position"].as_object().and_then(|p| p["y"].as_f64()),
    ) {
        Some(crate::engine::canvas_state::Position { x, y })
    } else {
        None
    };

    if let Some(st) = shape_type_str {
        // --- Geometric shape path ---
        let shape_type = crate::engine::canvas_state::ShapeType::from_str(st)
            .map_err(|e| format!("{}", e))?;

        // Get composite sub_shapes if applicable
        let (sub_shapes, override_size) =
            if crate::engine::shapes::is_composite(st) {
                crate::engine::shapes::get_composite_shapes(st)
                    .map(|(shapes, w, h)| (Some(shapes), Some((w, h))))
                    .unwrap_or((None, None))
            } else {
                (None, None)
            };

        let node_type = crate::engine::canvas_state::NodeType::Process; // placeholder for shape nodes

        let mut node = crate::engine::node_ops::add_node(
            &mut state.nodes, node_type, label, position, None,
        );

        // Apply shape_type and sub_shapes
        if let Some(n) = state.nodes.get_mut(&node.id) {
            n.shape_type = Some(shape_type);
            if let Some(shapes) = sub_shapes {
                n.sub_shapes = Some(shapes);
                if let Some((w, h)) = override_size {
                    n.size = crate::engine::canvas_state::Size { width: w, height: h };
                }
            }
            // Apply fill color from LLM args
            if let Some(fill) = args["fill"].as_str() {
                n.style.fill = fill.to_string();
            }
        }

        Ok(serde_json::json!({"node_id": node.id, "label": node.label, "shape_type": st}).to_string())
    } else {
        // --- Original flowchart path (unchanged) ---
        let node_type = crate::engine::canvas_state::NodeType::from_str(node_type_str)
            .map_err(|e| format!("{}", e))?;

        let node = crate::engine::node_ops::add_node(
            &mut state.nodes, node_type, label, position, None,
        );
        Ok(serde_json::json!({"node_id": node.id, "label": node.label}).to_string())
    }
}
```

Similarly update `"add_nodes_batch"`: after the existing batch creation and layout, iterate over created nodes and apply `shape_type` + `sub_shapes` for any nodes that have a `shape_type` in their args. Add this block after the layout code (before the `Ok(...)` return):

```rust
// After auto_layout in add_nodes_batch, apply shape_type for geometric shapes:
if let Some(nodes_arr) = args["nodes"].as_array() {
    for (i, node_arg) in nodes_arr.iter().enumerate() {
        if let Some(st) = node_arg["shape_type"].as_str() {
            if i < created.len() {
                let node_id = &created[i].id;
                if let Some(n) = state.nodes.get_mut(node_id) {
                    n.shape_type = Some(
                        crate::engine::canvas_state::ShapeType::from_str(st)
                            .unwrap_or(crate::engine::canvas_state::ShapeType::Rectangle),
                    );
                    if crate::engine::shapes::is_composite(st) {
                        if let Some((shapes, (w, h))) =
                            crate::engine::shapes::get_composite_shapes(st)
                        {
                            n.sub_shapes = Some(shapes);
                            n.size = crate::engine::canvas_state::Size { width: w, height: h };
                        }
                    }
                    if let Some(fill) = node_arg["fill"].as_str() {
                        n.style.fill = fill.to_string();
                    }
                }
            }
        }
    }
}
```

- [ ] **Step 2: Build Rust to verify compilation**

Run: `cd src-tauri && cargo build 2>&1`
Expected: Compiles with 0 errors.

- [ ] **Step 3: Run Rust tests**

Run: `cd src-tauri && cargo test 2>&1`
Expected: All tests pass (existing + new shapes tests).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/commands/mod.rs
git commit -m "feat(commands): handle shape_type in execute_tool_call

- add_node: check shape_type arg → ShapeType::from_str → populate
  node.shape_type + sub_shapes (composites) + size override + fill color
- add_nodes_batch: iterate created nodes, apply shape_type/sub_shapes
  for any node arg that has a shape_type field
- Original flowchart path (type arg) completely unchanged
- Both paths coexist in the same match arm via if-else"
```

---

### Task 8: End-to-End Build Verification

**Files:** (none — verification only)

- [ ] **Step 1: Full Rust build**

Run: `cd src-tauri && cargo build 2>&1`
Expected: 0 errors.

- [ ] **Step 2: Full TypeScript type-check**

Run: `cd /home/zuqis/voice-to-draw && npx tsc --noEmit 2>&1`
Expected: 0 errors.

- [ ] **Step 3: Run all Rust tests**

Run: `cd src-tauri && cargo test 2>&1`
Expected: All tests pass.

- [ ] **Step 4: Run full app build**

Run: `cd /home/zuqis/voice-to-draw && npm run build 2>&1`
Expected: Builds successfully.

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "chore: final verification — all builds and tests pass"
```

---

## Design Decisions

| 决策 | 选择 | 原因 |
|------|------|------|
| 类型独立 | `ShapeType` 新建，`NodeType` 不动 | 流程图和几何图形是不同的领域概念，独立 enum 语义清晰，零回归风险 |
| 复合图形存储 | 单节点 + `sub_shapes` 数组 (方案 A) | 移动/删除整体操作一步完成，子组件细节由 shapes.rs 集中管理 |
| 子组件数据位置 | Rust `engine/shapes.rs` 纯函数 | 后端控制业务逻辑（哪个组件什么颜色），前端纯渲染 |
| LLM 参数 | `shape_type` 和 `type` 并列可选 | LLM 根据语境选择正确参数，不会混淆流程图和几何图形 |
| 渲染分发 | `renderNode` 先检查 `shape_type`，早返回 | 流程图 switch 块一行不改，新增代码在前面独立执行 |
| 坐标口语化 | system prompt 教 LLM | 不需要硬编码，LLM 自然理解"左边""右上角"等表达 |

## Data Flow

```
用户语音: "画一座蓝色房子在左上角"
    ↓
preprocessor: denoise → quick_match (miss) → NeedsLLM
    ↓
LLM (system prompt 教过 shape_type + 坐标口语化):
    → tool_call: add_node { shape_type: "house", label: "房子",
                             fill: "#1e88e5", grid_x: 2, grid_y: 2 }
    ↓
execute_tool_call → is_composite("house") → get_composite_shapes("house")
    → DiagramNode { shape_type: Some(House),
                    sub_shapes: Some([roof, body, door, window]),
                    size: Size { width: 140, height: 150 }, ... }
    ↓
emit "canvas-updated" → frontend CanvasView.renderNode()
    → isCompositeShape("House") → renderCompositeShape(node)
    → fabric.Group([Triangle(roof), Rect(body), Rect(door), Rect(window)])
    ↓
画布显示房子 🏠
```

---

## Self-Review

1. **Spec coverage:**
   - [x] Basic geometric shapes — Task 1 (ShapeType) + Task 5 (ShapeRenderer) + Task 6 (wiring)
   - [x] Composite objects — Task 1 (shapes.rs recipes) + Task 5 (renderCompositeShape) + Task 7 (backend wiring)
   - [x] Coordinate colloquialisms — Task 3 (system prompt teaches LLM)
   - [x] Color recognition — Task 3 (color table in system prompt)
   - [x] Relative positioning — Task 3 (few-shot example 9)
   - [x] Attribute adjustments — Task 3 (few-shot example 10) + existing update_node
   - [x] Module separation — shapes.rs (composite definitions) + ShapeRenderer.ts (rendering), basic shapes inline

2. **Placeholder scan:** No TBD, TODO, or "implement later" found.

3. **Type consistency:**
   - `ShapeType` enum values match between Rust (`ShapeType::Circle`) and TypeScript (`"Circle"`) ✓
   - `SubShape` struct fields match between Rust and TypeScript ✓
   - `isCompositeShape("House")` matches `is_composite("house")` (case difference handled in TS predicate) ✓
   - `renderCompositeShape` reads `node.sub_shapes` which is populated in Task 7 ✓
   - `NodeType` unchanged on both sides ✓
