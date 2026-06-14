// 与 Rust CanvasState 对应的前端类型

// NodeType stays exactly as-is — flowchart types only
export type NodeType =
  | "Start"
  | "End"
  | "Process"
  | "Decision"
  | "Data"
  | "Subprocess"
  | "Text";

// NEW: ShapeType — geometric and composite shapes (parallel to NodeType)
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

export interface EdgeStyle {
  line_style: LineStyle;
  arrow: ArrowType;
  stroke: string;
  stroke_width: number;
}

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
  grid_size: number;
  grid_origin_x: number;
  grid_origin_y: number;
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
  pending_plan: OperationPlan | null;
}

/** 操作计划（预览确认用） */
export interface OperationPlan {
  id: string;
  diagram_type: string;
  summary: string;
  nodes: PlanNode[];
  edges: PlanEdge[];
  grid_position: [number, number] | null;
  layout_direction: "top_down" | "left_right";
  estimated_tool_calls: number;
}

export interface PlanNode {
  label: string;
  type: string;
}

export interface PlanEdge {
  from: string;
  to: string;
  label: string | null;
}

/** 对话消息 */
export interface ConversationMessage {
  role: "user" | "assistant";
  content: string;
}