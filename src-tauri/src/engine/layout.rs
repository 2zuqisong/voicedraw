use super::canvas_state::*;
use std::collections::HashMap;

/// 简单从上到下分层布局
/// 原理：BFS 遍历，按层分配 y，同层均匀分配 x
pub fn top_down_layout(
    nodes: &mut HashMap<String, DiagramNode>,
    edges: &HashMap<String, DiagramEdge>,
) {
    // 1. 找到起始节点（没有入边的节点）
    let has_incoming: std::collections::HashSet<&str> = edges
        .values()
        .map(|e| e.to_id.as_str())
        .collect();
    let roots: Vec<String> = nodes
        .keys()
        .filter(|id| !has_incoming.contains(id.as_str()))
        .cloned()
        .collect();

    if roots.is_empty() {
        // 没有明显根节点，按节点 ID 排序作为第一层
        layout_fallback(nodes);
        return;
    }

    // 2. BFS 分层
    let mut layers: Vec<Vec<String>> = vec![roots];
    let mut placed: std::collections::HashSet<String> =
        layers[0].iter().cloned().collect();

    loop {
        let prev_layer = &layers[layers.len() - 1];
        let mut next_layer: Vec<String> = Vec::new();
        for node_id in prev_layer {
            for edge in edges.values().filter(|e| e.from_id == *node_id) {
                if !placed.contains(&edge.to_id) {
                    next_layer.push(edge.to_id.clone());
                    placed.insert(edge.to_id.clone());
                }
            }
        }
        if next_layer.is_empty() {
            break;
        }
        layers.push(next_layer);
    }

    // 3. 将未遍历到的节点追加到最后一层
    for id in nodes.keys() {
        if !placed.contains(id) {
            if layers.is_empty() {
                layers.push(vec![id.clone()]);
            } else {
                layers.last_mut().unwrap().push(id.clone());
            }
            placed.insert(id.clone());
        }
    }

    // 4. 计算坐标（居中布置）
    let x_spacing = 200.0;
    let y_spacing = 120.0;
    let canvas_center_x = 580.0; // 画布水平中心（适配常见窗口宽度）
    let start_y = 80.0;

    for (layer_idx, layer) in layers.iter().enumerate() {
        let total_width = (layer.len() as f64 - 1.0) * x_spacing;
        let start_x = canvas_center_x - total_width / 2.0; // 居中
        for (node_idx, node_id) in layer.iter().enumerate() {
            if let Some(node) = nodes.get_mut(node_id) {
                node.position = Position {
                    x: start_x + node_idx as f64 * x_spacing,
                    y: start_y + layer_idx as f64 * y_spacing,
                };
            }
        }
    }
}

fn layout_fallback(nodes: &mut HashMap<String, DiagramNode>) {
    for (i, (_, node)) in nodes.iter_mut().enumerate() {
        node.position = Position {
            x: 480.0 + (i as f64 % 4.0) * 200.0,
            y: 80.0 + (i as f64 / 4.0).floor() * 120.0,
        };
    }
}

/// 从左到右分层布局（树状图/思维导图）
pub fn left_right_layout(
    nodes: &mut HashMap<String, DiagramNode>,
    edges: &HashMap<String, DiagramEdge>,
) {
    // 先执行 top_down 布局
    top_down_layout(nodes, edges);
    // 交换 x 和 y
    for node in nodes.values_mut() {
        std::mem::swap(&mut node.position.x, &mut node.position.y);
    }
}

/// 布局入口
pub enum LayoutDirection {
    TopDown,
    LeftRight,
}

pub fn auto_layout(
    nodes: &mut HashMap<String, DiagramNode>,
    edges: &HashMap<String, DiagramEdge>,
    direction: LayoutDirection,
) -> usize {
    let old_positions: HashMap<String, Position> = nodes
        .iter()
        .map(|(id, n)| (id.clone(), n.position.clone()))
        .collect();
    match direction {
        LayoutDirection::TopDown => top_down_layout(nodes, edges),
        LayoutDirection::LeftRight => left_right_layout(nodes, edges),
    }
    // 返回移动了的节点数
    nodes.iter().filter(|(id, n)| {
        old_positions.get(*id).map_or(true, |old| old.x != n.position.x || old.y != n.position.y)
    }).count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_top_down_three_nodes_chain() {
        let mut nodes = HashMap::new();
        let mut edges = HashMap::new();

        use crate::engine::node_ops::add_node;
        use crate::engine::edge_ops::add_edge;

        let n1 = add_node(&mut nodes, NodeType::Start, "开始".into(), None, None);
        let n2 = add_node(&mut nodes, NodeType::Process, "处理".into(), None, None);
        let n3 = add_node(&mut nodes, NodeType::End, "结束".into(), None, None);
        add_edge(&mut edges, &n1.id, &n2.id, None, None).unwrap();
        add_edge(&mut edges, &n2.id, &n3.id, None, None).unwrap();

        top_down_layout(&mut nodes, &edges);

        // 验证分层：n1 应该在最上面，n3 在最下面
        assert!(nodes[&n1.id].position.y < nodes[&n2.id].position.y);
        assert!(nodes[&n2.id].position.y < nodes[&n3.id].position.y);
    }
}