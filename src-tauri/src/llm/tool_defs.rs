use async_openai::types::{ChatCompletionTool, ChatCompletionToolType, FunctionObject};
use serde_json::json;

pub fn get_tool_definitions() -> Vec<ChatCompletionTool> {
    vec![
        tool(
            "add_nodes_batch",
            "批量添加节点到画布",
            json!({
                "type": "object",
                "properties": {
                    "nodes": {
                        "type": "array",
                        "description": "要添加的节点列表",
                        "items": {
                            "type": "object",
                            "properties": {
                                "type": {
                                    "type": "string",
                                    "enum": ["start", "end", "process", "decision", "data", "subprocess", "text"],
                                    "description": "节点类型"
                                },
                                "label": {
                                    "type": "string",
                                    "description": "节点显示的文本标签"
                                }
                            },
                            "required": ["type", "label"]
                        }
                    }
                },
                "required": ["nodes"]
            }),
        ),
        tool(
            "add_edges_batch",
            "批量添加连线",
            json!({
                "type": "object",
                "properties": {
                    "edges": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "from": {"type": "string", "description": "起始节点 ID"},
                                "to": {"type": "string", "description": "目标节点 ID"},
                                "label": {"type": "string", "description": "连线标签（可选）"}
                            },
                            "required": ["from", "to"]
                        }
                    }
                },
                "required": ["edges"]
            }),
        ),
        tool(
            "add_node",
            "添加单个节点",
            json!({
                "type": "object",
                "properties": {
                    "type": {"type": "string", "enum": ["start", "end", "process", "decision", "data", "subprocess", "text"]},
                    "label": {"type": "string"},
                    "position": {
                        "type": "object",
                        "properties": {
                            "x": {"type": "number"},
                            "y": {"type": "number"}
                        }
                    }
                },
                "required": ["type", "label"]
            }),
        ),
        tool(
            "add_edge",
            "添加单条连线",
            json!({
                "type": "object",
                "properties": {
                    "from": {"type": "string"},
                    "to": {"type": "string"},
                    "label": {"type": "string"}
                },
                "required": ["from", "to"]
            }),
        ),
        tool(
            "update_node",
            "修改节点属性",
            json!({
                "type": "object",
                "properties": {
                    "node_id": {"type": "string"},
                    "label": {"type": "string"},
                    "fill": {"type": "string", "description": "填充颜色，如 #e8f5e9"},
                    "stroke": {"type": "string", "description": "边框颜色"}
                },
                "required": ["node_id"]
            }),
        ),
        tool(
            "delete_node",
            "删除节点（自动删除关联连线）",
            json!({
                "type": "object",
                "properties": {
                    "node_id": {"type": "string"}
                },
                "required": ["node_id"]
            }),
        ),
        tool(
            "delete_edge",
            "删除连线",
            json!({
                "type": "object",
                "properties": {
                    "edge_id": {"type": "string"}
                },
                "required": ["edge_id"]
            }),
        ),
        tool(
            "auto_layout",
            "自动排列节点位置",
            json!({
                "type": "object",
                "properties": {
                    "direction": {
                        "type": "string",
                        "enum": ["top_down", "left_right"],
                        "description": "布局方向"
                    }
                }
            }),
        ),
        tool(
            "set_theme",
            "切换画布主题",
            json!({
                "type": "object",
                "properties": {
                    "theme": {
                        "type": "string",
                        "enum": ["default", "professional", "handdrawn", "dark", "colorful"]
                    }
                },
                "required": ["theme"]
            }),
        ),
        tool(
            "get_canvas_state",
            "获取当前画布状态",
            json!({
                "type": "object",
                "properties": {}
            }),
        ),
    ]
}

fn tool(
    name: &str,
    description: &str,
    parameters: serde_json::Value,
) -> ChatCompletionTool {
    ChatCompletionTool {
        function: FunctionObject {
            name: name.to_string(),
            description: Some(description.to_string()),
            parameters: Some(parameters),
            strict: None,
        },
        r#type: ChatCompletionToolType::Function,
    }
}
