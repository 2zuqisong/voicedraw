//! 预设模板系统 — 一键生成常见组合

use super::canvas_state::*;
use std::collections::HashMap;
use uuid::Uuid;

/// 模板定义
#[derive(Clone)]
pub struct Template {
    pub name: &'static str,
    pub description: &'static str,
    pub nodes: &'static [TemplateNode],
    pub edges: &'static [TemplateEdge],
}

#[derive(Clone)]
pub struct TemplateNode {
    pub shape_type: Option<&'static str>,
    pub node_type: Option<&'static str>,
    pub label: &'static str,
    pub rel_x: f64,
    pub rel_y: f64,
    pub fill: Option<&'static str>,
}

#[derive(Clone)]
pub struct TemplateEdge {
    pub from_label: &'static str,
    pub to_label: &'static str,
    pub label: Option<&'static str>,
    pub routing: Option<&'static str>,
}

/// 获取所有可用模板名称
pub fn get_template_names() -> Vec<&'static str> {
    ALL_TEMPLATES.iter().map(|t| t.name).collect()
}

/// 按名称查找模板
pub fn get_template(name: &str) -> Option<&'static Template> {
    ALL_TEMPLATES.iter().find(|t| t.name == name)
}

// ══════════════════════════════════════════════════════════════
// 模板定义
// ══════════════════════════════════════════════════════════════

const ALL_TEMPLATES: &[Template] = &[
    BIRTHDAY_CARD,
    WEATHER_SCENE,
    LOVE_CARD,
    FLOWCHART_STARTER,
    MIND_MAP_3,
];

const BIRTHDAY_CARD: Template = Template {
    name: "birthday_card",
    description: "生日贺卡：蛋糕居中 + 气球两侧 + 标题",
    nodes: &[
        TemplateNode { shape_type: Some("cake"), node_type: None, label: "", rel_x: 0.0, rel_y: 0.0, fill: Some("#ffab91") },
        TemplateNode { shape_type: Some("balloon"), node_type: None, label: "", rel_x: -150.0, rel_y: -20.0, fill: Some("#e53935") },
        TemplateNode { shape_type: Some("balloon"), node_type: None, label: "", rel_x: 150.0, rel_y: -20.0, fill: Some("#42a5f5") },
        TemplateNode { shape_type: None, node_type: Some("text"), label: "生日快乐", rel_x: 0.0, rel_y: -120.0, fill: None },
    ],
    edges: &[],
};

const WEATHER_SCENE: Template = Template {
    name: "weather_scene",
    description: "天气场景：太阳 + 云朵 + 闪电",
    nodes: &[
        TemplateNode { shape_type: Some("sun"), node_type: None, label: "", rel_x: 0.0, rel_y: 0.0, fill: Some("#ffc107") },
        TemplateNode { shape_type: Some("cloud"), node_type: None, label: "", rel_x: -180.0, rel_y: -20.0, fill: Some("#eceff1") },
        TemplateNode { shape_type: Some("cloud"), node_type: None, label: "", rel_x: 180.0, rel_y: -30.0, fill: Some("#cfd8dc") },
        TemplateNode { shape_type: Some("lightning"), node_type: None, label: "", rel_x: 220.0, rel_y: 40.0, fill: Some("#ffd600") },
    ],
    edges: &[],
};

const LOVE_CARD: Template = Template {
    name: "love_card",
    description: "爱心贺卡：心形居中 + 花朵两侧 + 标题",
    nodes: &[
        TemplateNode { shape_type: Some("heart"), node_type: None, label: "", rel_x: 0.0, rel_y: 0.0, fill: Some("#e53935") },
        TemplateNode { shape_type: Some("flower"), node_type: None, label: "", rel_x: -140.0, rel_y: 0.0, fill: Some("#f48fb1") },
        TemplateNode { shape_type: Some("flower"), node_type: None, label: "", rel_x: 140.0, rel_y: 0.0, fill: Some("#ce93d8") },
        TemplateNode { shape_type: None, node_type: Some("text"), label: "I ❤ U", rel_x: 0.0, rel_y: -120.0, fill: None },
    ],
    edges: &[],
};

const FLOWCHART_STARTER: Template = Template {
    name: "flowchart_starter",
    description: "基础流程图：开始→处理→结束",
    nodes: &[
        TemplateNode { shape_type: None, node_type: Some("start"), label: "开始", rel_x: 0.0, rel_y: 0.0, fill: None },
        TemplateNode { shape_type: None, node_type: Some("process"), label: "处理", rel_x: 0.0, rel_y: 120.0, fill: None },
        TemplateNode { shape_type: None, node_type: Some("end"), label: "结束", rel_x: 0.0, rel_y: 240.0, fill: None },
    ],
    edges: &[
        TemplateEdge { from_label: "开始", to_label: "处理", label: None, routing: Some("orthogonal") },
        TemplateEdge { from_label: "处理", to_label: "结束", label: None, routing: Some("orthogonal") },
    ],
};

const MIND_MAP_3: Template = Template {
    name: "mind_map_3",
    description: "思维导图：1 中心 + 3 分支",
    nodes: &[
        TemplateNode { shape_type: None, node_type: Some("subprocess"), label: "中心主题", rel_x: 0.0, rel_y: 0.0, fill: None },
        TemplateNode { shape_type: None, node_type: Some("process"), label: "分支 1", rel_x: -200.0, rel_y: -100.0, fill: None },
        TemplateNode { shape_type: None, node_type: Some("process"), label: "分支 2", rel_x: 200.0, rel_y: -100.0, fill: None },
        TemplateNode { shape_type: None, node_type: Some("process"), label: "分支 3", rel_x: 0.0, rel_y: 120.0, fill: None },
    ],
    edges: &[
        TemplateEdge { from_label: "中心主题", to_label: "分支 1", label: None, routing: None },
        TemplateEdge { from_label: "中心主题", to_label: "分支 2", label: None, routing: None },
        TemplateEdge { from_label: "中心主题", to_label: "分支 3", label: None, routing: None },
    ],
};

// ══════════════════════════════════════════════════════════════
// 模板实例化：生成 DiagramNode + DiagramEdge
// ══════════════════════════════════════════════════════════════

/// 将模板实例化为画布节点和连线
/// origin_x, origin_y: 模板左上角在画布上的像素坐标
/// title_override: 覆盖模板中的 text 类型节点 label
pub fn instantiate_template(
    template: &Template,
    origin_x: f64,
    origin_y: f64,
    title_override: Option<&str>,
) -> (Vec<DiagramNode>, Vec<DiagramEdge>) {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    // label → node_id 映射（用于解析边）
    let mut label_to_id: HashMap<String, String> = HashMap::new();

    for tn in template.nodes {
        let id = Uuid::new_v4().to_string();
        let label = if tn.node_type == Some("text") {
            title_override.unwrap_or(tn.label).to_string()
        } else {
            tn.label.to_string()
        };

        let node_type = match tn.node_type {
            Some("start") => NodeType::Start,
            Some("end") => NodeType::End,
            Some("process") => NodeType::Process,
            Some("decision") => NodeType::Decision,
            Some("data") => NodeType::Data,
            Some("subprocess") => NodeType::Subprocess,
            Some("text") => NodeType::Text,
            _ => NodeType::Process,
        };

        let shape_type = tn.shape_type.and_then(|s| ShapeType::from_str(s).ok());

        let (sub_shapes, size) = if let Some(ref st_str) = tn.shape_type {
            if super::shapes::is_composite(st_str) {
                let fill_opt = tn.fill;
                super::shapes::get_composite_shapes(st_str, fill_opt)
                    .map(|(shapes, w, h)| (Some(shapes), Size { width: w, height: h }))
                    .unwrap_or((None, Size { width: 160.0, height: 60.0 }))
            } else {
                (None, Size { width: 160.0, height: 60.0 })
            }
        } else {
            (None, Size { width: 160.0, height: 60.0 })
        };

        let mut style = NodeStyle::default();
        if let Some(f) = tn.fill {
            style.fill = f.to_string();
        }

        let node = DiagramNode {
            id: id.clone(),
            node_type,
            shape_type,
            label: label.clone(),
            position: Position {
                x: origin_x + tn.rel_x,
                y: origin_y + tn.rel_y,
            },
            size,
            style,
            sub_shapes,
        };

        if !tn.label.is_empty() {
            label_to_id.insert(tn.label.to_string(), id.clone());
        }
        // 对于 text 类型，也用 label 作为 key
        if tn.node_type == Some("text") {
            label_to_id.insert(label.clone(), id.clone());
        }
        nodes.push(node);
    }

    for te in template.edges {
        let from_id = label_to_id.get(te.from_label);
        let to_id = label_to_id.get(te.to_label);
        if let (Some(fid), Some(tid)) = (from_id, to_id) {
            let routing = match te.routing {
                Some("orthogonal") => RoutingMode::Orthogonal,
                _ => RoutingMode::Straight,
            };
            let edge = DiagramEdge {
                id: Uuid::new_v4().to_string(),
                from_id: fid.clone(),
                to_id: tid.clone(),
                label: te.label.map(|s| s.to_string()),
                style: EdgeStyle {
                    routing,
                    ..Default::default()
                },
                waypoints: None,
            };
            edges.push(edge);
        }
    }

    (nodes, edges)
}
