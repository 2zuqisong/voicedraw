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