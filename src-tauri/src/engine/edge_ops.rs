use super::canvas_state::*;
use std::collections::HashMap;
use uuid::Uuid;

/// 添加连线
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
