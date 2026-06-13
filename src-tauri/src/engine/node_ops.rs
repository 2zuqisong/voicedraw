use super::canvas_state::*;
use std::collections::HashMap;
use uuid::Uuid;

/// 添加单个节点
pub fn add_node(
    nodes: &mut HashMap<String, DiagramNode>,
    node_type: NodeType,
    label: String,
    position: Option<Position>,
    style: Option<NodeStyle>,
) -> DiagramNode {
    let id = Uuid::new_v4().to_string();
    let pos = position.unwrap_or(Position { x: 740.0, y: 424.0 });
    let node = DiagramNode {
        id: id.clone(),
        node_type,
        label,
        position: pos,
        size: Size { width: 160.0, height: 60.0 },
        style: style.unwrap_or_default(),
    };
    nodes.insert(id, node.clone());
    node
}

/// 批量添加节点（位置由自动布局决定时为 None）
pub fn add_nodes_batch(
    nodes: &mut HashMap<String, DiagramNode>,
    batch: Vec<(NodeType, String)>,
) -> Vec<DiagramNode> {
    batch
        .into_iter()
        .map(|(nt, label)| add_node(nodes, nt, label, None, None))
        .collect()
}

/// 更新节点
pub fn update_node(
    nodes: &mut HashMap<String, DiagramNode>,
    node_id: &str,
    label: Option<String>,
    style: Option<NodeStyle>,
    position: Option<Position>,
) -> Result<DiagramNode, String> {
    let node = nodes.get_mut(node_id).ok_or(format!("节点 {} 不存在", node_id))?;
    if let Some(l) = label { node.label = l; }
    if let Some(s) = style { node.style = s; }
    if let Some(p) = position { node.position = p; }
    Ok(node.clone())
}

/// 删除节点（返回被删除的关联连线 from_id/to_id）
pub fn delete_node(
    nodes: &mut HashMap<String, DiagramNode>,
    edges: &mut HashMap<String, DiagramEdge>,
    node_id: &str,
) -> Result<(DiagramNode, Vec<String>), String> {
    let removed = nodes.remove(node_id).ok_or(format!("节点 {} 不存在", node_id))?;
    let deleted_edges: Vec<String> = edges
        .iter()
        .filter(|(_, e)| e.from_id == node_id || e.to_id == node_id)
        .map(|(id, _)| id.clone())
        .collect();
    for id in &deleted_edges {
        edges.remove(id);
    }
    Ok((removed, deleted_edges))
}

/// 移动节点（支持相对方向）
pub fn move_node(
    nodes: &mut HashMap<String, DiagramNode>,
    node_id: &str,
    target: MoveTarget,
) -> Result<(Position, Position), String> {
    let node = nodes.get_mut(node_id).ok_or(format!("节点 {} 不存在", node_id))?;
    let old = node.position.clone();
    match target {
        MoveTarget::Absolute(x, y) => { node.position = Position { x, y }; }
        MoveTarget::Direction(dir) => {
            let step = 60.0;
            match dir.as_str() {
                "left" => node.position.x -= step,
                "right" => node.position.x += step,
                "up" => node.position.y -= step,
                "down" => node.position.y += step,
                _ => return Err(format!("未知方向: {}", dir)),
            }
        }
    }
    Ok((old, node.position.clone()))
}

pub enum MoveTarget {
    Absolute(f64, f64),
    Direction(String),
}

/// 获取画布上的节点列表摘要（不含完整样式细节，给 LLM 用的轻量信息）
pub fn get_nodes_summary(nodes: &HashMap<String, DiagramNode>) -> Vec<NodeSummary> {
    nodes.values().map(|n| NodeSummary {
        id: n.id.clone(),
        node_type: format!("{:?}", n.node_type),
        label: n.label.clone(),
        x: n.position.x,
        y: n.position.y,
    }).collect()
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NodeSummary {
    pub id: String,
    pub node_type: String,
    pub label: String,
    pub x: f64,
    pub y: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_delete_node() {
        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();
        let node = add_node(&mut nodes, NodeType::Process, "测试节点".into(), None, None);
        assert_eq!(nodes.len(), 1);
        
        let (removed, deleted) = delete_node(&mut nodes, &mut edges, &node.id).unwrap();
        assert_eq!(removed.label, "测试节点");
        assert!(deleted.is_empty());
        assert!(nodes.is_empty());
    }

    #[test]
    fn test_delete_node_removes_edges() {
        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();
        let n1 = add_node(&mut nodes, NodeType::Start, "A".into(), None, None);
        let n2 = add_node(&mut nodes, NodeType::End, "B".into(), None, None);
        let e = crate::engine::edge_ops::add_edge(&mut edges, &n1.id, &n2.id, None, None).unwrap();
        
        let (_, deleted) = delete_node(&mut nodes, &mut edges, &n1.id).unwrap();
        assert!(deleted.contains(&e.id));
        assert!(edges.is_empty());
    }
}