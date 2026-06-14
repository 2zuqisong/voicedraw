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
            font_family: "Noto Sans SC, Microsoft YaHei, sans-serif".into(),
            border_radius: 4.0,
            opacity: 1.0,
            text_color: "#1a237e".into(),
            shadow: None,
        },
        Theme::Handdrawn => NodeStyle {
            fill: "#fffde7".into(),
            stroke: "#5d4037".into(),
            stroke_width: 2.5,
            font_size: 15.0,
            font_family: "Comic Sans MS, cursive".into(),
            border_radius: 12.0,
            opacity: 0.95,
            text_color: "#3e2723".into(),
            shadow: Some(ShadowConfig {
                color: "rgba(0,0,0,0.08)".into(),
                blur: 3.0,
                offset_x: 1.0,
                offset_y: 1.0,
            }),
        },
        Theme::Dark => NodeStyle {
            fill: "#37474f".into(),
            stroke: "#78909c".into(),
            stroke_width: 2.0,
            font_size: 14.0,
            font_family: "Noto Sans SC, sans-serif".into(),
            border_radius: 6.0,
            opacity: 1.0,
            text_color: "#eceff1".into(),
            shadow: None,
        },
        Theme::Colorful => NodeStyle {
            fill: "#fff3e0".into(),
            stroke: "#e65100".into(),
            stroke_width: 2.5,
            font_size: 14.0,
            font_family: "Noto Sans SC, sans-serif".into(),
            border_radius: 8.0,
            opacity: 1.0,
            text_color: "#bf360c".into(),
            shadow: Some(ShadowConfig {
                color: "rgba(230,81,0,0.15)".into(),
                blur: 6.0,
                offset_x: 0.0,
                offset_y: 2.0,
            }),
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
    stroke_width: Option<f64>,
    border_radius: Option<f64>,
    text_color: Option<String>,
    opacity: Option<f64>,
) -> Result<NodeStyle, String> {
    let node = nodes.get_mut(target_id).ok_or(format!("节点 {} 不存在", target_id))?;
    if let Some(f) = fill { node.style.fill = f; }
    if let Some(s) = stroke { node.style.stroke = s; }
    if let Some(fs) = font_size { node.style.font_size = fs; }
    if let Some(sw) = stroke_width { node.style.stroke_width = sw; }
    if let Some(br) = border_radius { node.style.border_radius = br; }
    if let Some(tc) = text_color { node.style.text_color = tc; }
    if let Some(o) = opacity { node.style.opacity = o.clamp(0.0, 1.0); }
    Ok(node.style.clone())
}