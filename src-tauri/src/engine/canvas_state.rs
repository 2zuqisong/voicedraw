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

/// 几何图形类型（与流程图 NodeType 并列，互不干扰）
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ShapeType {
    // 基本几何图形
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
    Cake,
    Gift,
    Balloon,
    Candle,
    Heart,
    Flower,
    ArrowShape,
    SpeechBubble,
    Cloud,
    Lightning,
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
            "cake" => Ok(Self::Cake),
            "gift" => Ok(Self::Gift),
            "balloon" => Ok(Self::Balloon),
            "candle" => Ok(Self::Candle),
            "heart" => Ok(Self::Heart),
            "flower" => Ok(Self::Flower),
            "arrow_shape" => Ok(Self::ArrowShape),
            "speech_bubble" => Ok(Self::SpeechBubble),
            "cloud" => Ok(Self::Cloud),
            "lightning" => Ok(Self::Lightning),
            _ => Err(format!("未知图形类型: {}", s)),
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

/// 阴影配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowConfig {
    pub color: String,
    pub blur: f64,
    pub offset_x: f64,
    pub offset_y: f64,
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
    /// 不透明度 0.0–1.0（默认 1.0）
    #[serde(default = "default_opacity")]
    pub opacity: f64,
    /// 文字颜色（默认 #1a1a1a）
    #[serde(default = "default_text_color")]
    pub text_color: String,
    /// 可选阴影
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shadow: Option<ShadowConfig>,
}

fn default_opacity() -> f64 {
    1.0
}
fn default_text_color() -> String {
    "#1a1a1a".into()
}

impl Default for NodeStyle {
    fn default() -> Self {
        Self {
            fill: "#ffffff".into(),
            stroke: "#333333".into(),
            stroke_width: 2.0,
            font_size: 14.0,
            font_family: "Noto Sans SC, Microsoft YaHei, sans-serif".into(),
            border_radius: 8.0,
            opacity: 1.0,
            text_color: "#1a1a1a".into(),
            shadow: None,
        }
    }
}

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

/// 连线样式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeStyle {
    pub line_style: LineStyle,
    pub arrow: ArrowType,
    pub routing: RoutingMode,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoutingMode {
    Straight,
    Orthogonal,
}

impl Default for EdgeStyle {
    fn default() -> Self {
        Self {
            line_style: LineStyle::Solid,
            arrow: ArrowType::Single,
            routing: RoutingMode::Straight,
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

/// 图表连线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagramEdge {
    pub id: String,
    pub from_id: String,
    pub to_id: String,
    pub label: Option<String>,
    pub style: EdgeStyle,
    /// 正交路由的拐点（仅 routing=Orthogonal 时使用；None 时前端自动计算）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub waypoints: Option<Vec<Position>>,
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

/// 待前端执行的异步操作（如风格转换，需要前端先捕获 canvas 图像）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingAction {
    /// 操作类型: "apply_style"
    pub action_type: String,
    /// 风格转换 prompt
    pub prompt: String,
    /// 目标节点 ID 列表（空 = 整个画布）
    pub node_ids: Vec<String>,
}

/// 像素画布数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PixelCanvas {
    /// "row,col" → hex color string（稀疏存储，只记有颜色的格子）
    pub cells: HashMap<String, String>,
    /// 每格像素大小
    pub cell_size: u32,
    /// 列数
    pub cols: u32,
    /// 行数
    pub rows: u32,
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
    pub grid_size: f64,
    pub grid_origin_x: f64,
    pub grid_origin_y: f64,
    /// 像素画布（仅像素模式使用）
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pixel: Option<PixelCanvas>,
}