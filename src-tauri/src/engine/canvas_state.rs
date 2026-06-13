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
    pub grid_size: f64,
    pub grid_origin_x: f64,
    pub grid_origin_y: f64,
}