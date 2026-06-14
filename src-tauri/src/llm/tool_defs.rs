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
                                    "description": "流程图节点类型（与 shape_type 二选一）"
                                },
                                "shape_type": {
                                    "type": "string",
                                    "enum": ["circle", "rectangle", "triangle", "line", "dot", "house", "sun", "tree", "smiley", "star", "cake", "gift", "balloon", "candle", "heart", "flower", "arrow_shape", "speech_bubble", "cloud", "lightning"],
                                    "description": "几何图形类型（与 type 二选一，画几何图形时用此字段）"
                                },
                                "label": {
                                    "type": "string",
                                    "description": "节点显示的文本标签"
                                }
                            },
                            "required": ["label"]
                        }
                    },
                    "grid_x": {
                        "type": "number",
                        "description": "网格 X 坐标（可选，不填则自动找空位）。1格=20像素"
                    },
                    "grid_y": {
                        "type": "number",
                        "description": "网格 Y 坐标（可选，不填则自动找空位）。1格=20像素"
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
                                "label": {"type": "string", "description": "连线标签（可选）"},
                                "routing": {"type": "string", "enum": ["straight", "orthogonal"], "description": "路由模式：straight=直线，orthogonal=直角折线"}
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
                    "type": {"type": "string", "enum": ["start", "end", "process", "decision", "data", "subprocess", "text"], "description": "流程图节点类型（与 shape_type 二选一）"},
                    "shape_type": {"type": "string", "enum": ["circle", "rectangle", "triangle", "line", "dot", "house", "sun", "tree", "smiley", "star", "cake", "gift", "balloon", "candle", "heart", "flower", "arrow_shape", "speech_bubble", "cloud", "lightning"], "description": "几何图形类型（与 type 二选一，画几何图形时用此字段）"},
                    "label": {"type": "string"},
                    "position": {
                        "type": "object",
                        "properties": {
                            "x": {"type": "number"},
                            "y": {"type": "number"}
                        }
                    },
                    "grid_x": {
                        "type": "number",
                        "description": "网格 X 坐标（可选，不填则自动找空位）。1格=20像素"
                    },
                    "grid_y": {
                        "type": "number",
                        "description": "网格 Y 坐标（可选，不填则自动找空位）。1格=20像素"
                    }
                },
                "required": ["label"]
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
                    "label": {"type": "string"},
                    "routing": {"type": "string", "enum": ["straight", "orthogonal"], "description": "路由模式"}
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
                    "stroke": {"type": "string", "description": "边框颜色"},
                    "font_size": {"type": "number", "description": "字体大小"},
                    "stroke_width": {"type": "number", "description": "边框粗细"},
                    "border_radius": {"type": "number", "description": "圆角半径，0为直角"},
                    "text_color": {"type": "string", "description": "文字颜色，如 #333333"},
                    "opacity": {"type": "number", "description": "不透明度 0.0-1.0，1.0为完全不透明"}
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
        tool(
            "get_empty_anchor",
            "获取画布上当前未被占用的推荐锚点坐标，用于放置新图表",
            json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        ),
        tool(
            "apply_style",
            "将画布上的图形进行艺术风格转换。用户说'变成XX风格'/'切换成XX画风'/'转成XX效果'时调用。预设风格：梵高星空/莫奈印象/浮世绘/水墨画/像素风/素描。",
            json!({
                "type": "object",
                "properties": {
                    "prompt": {
                        "type": "string",
                        "description": "风格描述提示词（中文），如'梵高星空风格油画，强烈漩涡笔触，深蓝夜空，亮黄星星，厚涂纹理'。LLM 应自动将简单的风格名称扩展为详细的艺术描述。"
                    },
                    "node_ids": {
                        "type": "array",
                        "items": {"type": "string"},
                        "description": "要应用风格的目标节点 ID 列表。从对话历史中匹配目标节点。省略则应用于整个画布。"
                    }
                },
                "required": ["prompt"]
            }),
        ),
        tool(
            "apply_template",
            "应用预设模板一键生成组合。用户说'画贺卡'/'画天气'/'画思维导图'/'画基础流程'时调用。",
            json!({
                "type": "object",
                "properties": {
                    "template": {
                        "type": "string",
                        "enum": ["birthday_card", "weather_scene", "love_card", "flowchart_starter", "mind_map_3"],
                        "description": "模板名称"
                    },
                    "grid_x": {"type": "number", "description": "网格 X 坐标"},
                    "grid_y": {"type": "number", "description": "网格 Y 坐标"},
                    "title": {"type": "string", "description": "覆盖模板中的默认标题文字"}
                },
                "required": ["template"]
            }),
        ),
        // ── 像素绘画工具 ─────────────────────────────────────────────
        tool(
            "pixel_emoji",
            "在像素画布上放置表情小人。用户说'画笑脸'/'发一个笑哭'/'放一个爱心眼'时调用。注意：每个表情占 16×16 格子。",
            json!({
                "type": "object",
                "properties": {
                    "emoji": {
                        "type": "string",
                        "enum": ["smile", "laugh", "heart_eyes", "angry", "cry", "cool", "shock", "wink"],
                        "description": "表情名称：smile=笑脸, laugh=笑哭, heart_eyes=爱心眼, angry=生气, cry=哭泣, cool=酷墨镜, shock=惊讶, wink=眨眼吐舌"
                    },
                    "row": {"type": "integer", "description": "表情左上角行号（0-16）。建议放左边用0，右边用0或16"},
                    "col": {"type": "integer", "description": "表情左上角列号（0-16）。放左边用0，右边用16"}
                },
                "required": ["emoji", "row", "col"]
            }),
        ),
        tool(
            "pixel_set",
            "设置像素格子颜色。用于在像素画布上画点、线或区域。可以一次设置多个格子，每个格子指定行列和颜色。颜色为 hex 格式如 #ff0000。不指定颜色则擦除该格子。",
            json!({
                "type": "object",
                "properties": {
                    "cells": {
                        "type": "array",
                        "description": "要设置的格子列表",
                        "items": {
                            "type": "object",
                            "properties": {
                                "row": {"type": "integer", "description": "行号（从0开始，上方为0）"},
                                "col": {"type": "integer", "description": "列号（从0开始，左侧为0）"},
                                "color": {"type": "string", "description": "hex 颜色如 #ff0000；不填则擦除"}
                            },
                            "required": ["row", "col"]
                        }
                    }
                },
                "required": ["cells"]
            }),
        ),
        tool(
            "pixel_fill",
            "像素画布填充桶。将指定位置开始的连通区域填充为指定颜色。类似画图软件的油漆桶工具。",
            json!({
                "type": "object",
                "properties": {
                    "row": {"type": "integer", "description": "起始行号"},
                    "col": {"type": "integer", "description": "起始列号"},
                    "color": {"type": "string", "description": "填充颜色 hex，如 #ffffff"}
                },
                "required": ["row", "col", "color"]
            }),
        ),
        tool(
            "pixel_rect",
            "在像素画布上画一个实心矩形。",
            json!({
                "type": "object",
                "properties": {
                    "row": {"type": "integer", "description": "矩形左上角行号"},
                    "col": {"type": "integer", "description": "矩形左上角列号"},
                    "width": {"type": "integer", "description": "矩形宽度（列数）"},
                    "height": {"type": "integer", "description": "矩形高度（行数）"},
                    "color": {"type": "string", "description": "填充颜色 hex，如 #ff0000"}
                },
                "required": ["row", "col", "width", "height", "color"]
            }),
        ),
        tool(
            "pixel_clear",
            "清空像素画布上的所有内容。",
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
