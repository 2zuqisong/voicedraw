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
}

/** 对话消息 */
export interface ConversationMessage {
  role: "user" | "assistant";
  content: string;
}