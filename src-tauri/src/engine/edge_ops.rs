use super::canvas_state::*;
use std::collections::HashMap;
use uuid::Uuid;

/// 添加单条连线
pub fn add_edge(
    edges: &mut HashMap<String, DiagramEdge>,
    from_id: &str,
    to_id: &str,
    label: Option<String>,
    style: Option<EdgeStyle>,
) -> Result<DiagramEdge, String> {
    let id = Uuid::new_v4().to_string();
    let edge = DiagramEdge {
        id: id.clone(),
        from_id: from_id.to_string(),
        to_id: to_id.to_string(),
        label,
        style: style.unwrap_or_default(),
    };
    edges.insert(id, edge.clone());
    Ok(edge)
}

/// 批量添加连线
pub fn add_edges_batch(
    edges: &mut HashMap<String, DiagramEdge>,
    batch: Vec<EdgeDef>,
) -> Vec<DiagramEdge> {
    batch
        .into_iter()
        .filter_map(|def| add_edge(edges, &def.from, &def.to, def.label, def.style).ok())
        .collect()
}

pub struct EdgeDef {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub style: Option<EdgeStyle>,
}

/// 更新连线
pub fn update_edge(
    edges: &mut HashMap<String, DiagramEdge>,
    edge_id: &str,
    label: Option<String>,
    style: Option<EdgeStyle>,
) -> Result<DiagramEdge, String> {
    let edge = edges.get_mut(edge_id).ok_or(format!("连线 {} 不存在", edge_id))?;
    if let Some(l) = label { edge.label = Some(l); }
    if let Some(s) = style { edge.style = s; }
    Ok(edge.clone())
}

/// 删除连线
pub fn delete_edge(
    edges: &mut HashMap<String, DiagramEdge>,
    edge_id: &str,
) -> Result<DiagramEdge, String> {
    edges.remove(edge_id).ok_or(format!("连线 {} 不存在", edge_id))
}