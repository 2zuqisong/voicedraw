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
  | "Star"
  | "Cake"
  | "Gift"
  | "Balloon"
  | "Candle"
  | "Heart"
  | "Flower"
  | "ArrowShape"
  | "SpeechBubble"
  | "Cloud"
  | "Lightning";

export type Theme =
  | "Default"
  | "Professional"
  | "Handdrawn"
  | "Dark"
  | "Colorful";

export type LineStyle = "Solid" | "Dashed" | "Dotted";
export type ArrowType = "Single" | "Double" | "None";
export type RoutingMode = "Straight" | "Orthogonal";
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
  /** 不透明度 0.0–1.0（默认 1.0） */
  opacity: number;
  /** 文字颜色（默认 #1a1a1a） */
  text_color: string;
  /** 可选阴影 */
  shadow?: ShadowConfig;
}

/** 阴影配置 */
export interface ShadowConfig {
  color: string;
  blur: number;
  offset_x: number;
  offset_y: number;
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
  routing: RoutingMode;
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
  waypoints?: Position[] | null;
}

/** 像素画布数据（与 Rust PixelCanvas 对应） */
export interface PixelCanvas {
  cells: Record<string, string>; // "row,col" → hex
  cell_size: number;
  cols: number;
  rows: number;
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
  /** 像素画布（像素模式使用） */
  pixel?: PixelCanvas | null;
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
  pending_action: PendingAction | null;
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

/** 待前端执行的异步操作（如风格转换需捕获 canvas 图像） */
export interface PendingAction {
  action_type: string;
  prompt: string;
  node_ids: string[];
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

/** 风格转换请求（前端 → Rust） */
export interface StyleTransferRequest {
  image_base64: string;
  prompt: string;
  node_ids: string[];
}

/** 风格转换结果（Rust → 前端） */
export interface StyleTransferResult {
  image_base64: string;
  /** 原节点 ID 列表（前端需移除这些节点） */
  replaced_node_ids: string[];
}

// ── 可扩展设置模型 ────────────────────────────────────────────

/** 单个 API 厂商配置 */
export interface ProviderConfig {
  id: string;
  name: string;
  api_key: string;
  endpoint: string;
  model: string;
}

/** 厂商组（LLM 或图像生成） */
export interface ProviderGroup {
  /** 当前激活的厂商 ID */
  active: string;
  /** 所有可用厂商，key 为厂商 ID */
  providers: Record<string, ProviderConfig>;
}

/** 应用设置（持久化到 localStorage） */
export interface AppSettings {
  llm: ProviderGroup;
  image: ProviderGroup;
}

// ── 像素模式 ────────────────────────────────────────────────────────

export type CanvasMode = "vector" | "pixel";

export type PixelTool = "pencil" | "eraser" | "fill" | "picker";

/** 像素画布状态 */
export interface PixelState {
  /** "row,col" → hex 颜色 */
  data: Record<string, string>;
  /** 当前画笔颜色 */
  color: string;
  /** 当前工具 */
  tool: PixelTool;
  /** 格子像素大小 */
  cellSize: number;
  /** 网格列数 */
  cols: number;
  /** 网格行数 */
  rows: number;
  /** 撤销栈 */
  undoStack: Record<string, string>[];
}