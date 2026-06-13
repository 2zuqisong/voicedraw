use async_openai::types::{
    ChatCompletionMessageToolCall, ChatCompletionRequestAssistantMessageArgs,
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessageArgs,
    ChatCompletionRequestToolMessageArgs, ChatCompletionRequestUserMessageArgs,
    FunctionCall,
};
use serde_json::Value;

use super::deepseek_client::DeepSeekClient;
use super::system_prompt::get_system_prompt;
use super::tool_defs::get_tool_definitions;
use crate::engine::AppEngine;

/// LLM 调度器：管理 DeepSeek API 多轮对话循环
pub struct LLMScheduler {
    client: DeepSeekClient,
    max_rounds: u8,
}

impl LLMScheduler {
    pub fn new(api_key: String) -> Self {
        Self {
            client: DeepSeekClient::new(api_key, None),
            max_rounds: 5,
        }
    }

    /// 处理用户指令，返回最终回复和更新的 Canvas 状态
    pub async fn process(
        &self,
        user_text: &str,
        history: &[(String, String)], // (role, content)
        engine: &AppEngine,
    ) -> Result<SchedulerResult, String> {
        log::info!(
            "LLM Scheduler: 处理指令 '{}', 历史 {} 轮",
            user_text,
            history.len() / 2
        );

        let mut messages: Vec<ChatCompletionRequestMessage> = Vec::new();

        // 1. System prompt
        messages.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(get_system_prompt())
                .build()
                .map_err(|e| format!("构建 system message 失败: {}", e))?
                .into(),
        );

        // 2. 历史对话（最近 5 轮）
        for (role, content) in history.iter().rev().take(5).rev() {
            match role.as_str() {
                "user" => {
                    messages.push(
                        ChatCompletionRequestUserMessageArgs::default()
                            .content(content.clone())
                            .build()
                            .map_err(|e| format!("构建 user message 失败: {}", e))?
                            .into(),
                    );
                }
                "assistant" => {
                    messages.push(
                        ChatCompletionRequestAssistantMessageArgs::default()
                            .content(content.clone())
                            .build()
                            .map_err(|e| format!("构建 assistant message 失败: {}", e))?
                            .into(),
                    );
                }
                _ => {}
            }
        }

        // 3. 对话历史摘要
        let conv_summary = if history.is_empty() {
            "（无历史对话）".into()
        } else {
            history
                .iter()
                .enumerate()
                .filter_map(|(_i, (role, content))| {
                    if role == "user" {
                        Some(format!("用户: {}", content))
                    } else {
                        Some(format!("助手: {}", content))
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        // 4. 当前 canvas 状态摘要（作为上下文注入）
        let canvas_summary = {
            let canvas = engine.canvas.lock().unwrap();
            canvas.as_ref().map(|c| {
                format!(
                    "当前画布: {}, 节点数: {}, 连线数: {}, 主题: {:?}",
                    c.title,
                    c.nodes.len(),
                    c.edges.len(),
                    c.theme
                )
            }).unwrap_or_default()
        };

        let user_msg = format!(
            "对话历史:\n{}\n\n用户指令: {}\n{}",
            conv_summary,
            user_text,
            if canvas_summary.is_empty() {
                "画布为空".into()
            } else {
                canvas_summary
            }
        );

        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_msg)
                .build()
                .map_err(|e| format!("构建 user message 失败: {}", e))?
                .into(),
        );

        // 5. 工具定义
        let tools = get_tool_definitions();

        let mut final_content = String::new();

        // 6. 多轮循环（最多 max_rounds 轮）
        for round in 0..self.max_rounds {
            log::info!("LLM 第 {}/{} 轮...", round + 1, self.max_rounds);

            let response = self
                .client
                .chat(messages.clone(), tools.clone(), false)
                .await?;

            // 检查是否有 tool_calls
            let has_tool_calls = response
                .tool_calls
                .as_ref()
                .map(|tc| !tc.is_empty())
                .unwrap_or(false);

            if !has_tool_calls {
                // LLM 返回了纯文本回复，对话结束
                final_content = response.content.unwrap_or_else(|| "操作完成".into());
                log::info!("LLM 返回纯文本回复，结束循环: {}", final_content);
                break;
            }

            // 获取 tool_calls（此时已确认有值）
            let tool_calls = response.tool_calls.as_ref().unwrap();
            log::info!(
                "第 {} 轮 LLM 请求 {} 个工具调用: {:?}",
                round + 1,
                tool_calls.len(),
                tool_calls.iter().map(|tc| tc.name.as_str()).collect::<Vec<_>>()
            );

            // 将 assistant 消息（含 tool_calls）添加到对话历史
            // 将 DeepSeekClient 返回的 ToolCall 转换为 async_openai 的 ChatCompletionMessageToolCall
            let openai_tool_calls: Vec<ChatCompletionMessageToolCall> = tool_calls
                .iter()
                .map(|tc| ChatCompletionMessageToolCall {
                    id: tc.id.clone(),
                    r#type: async_openai::types::ChatCompletionToolType::Function,
                    function: FunctionCall {
                        name: tc.name.clone(),
                        arguments: tc.arguments.clone(),
                    },
                })
                .collect();

            messages.push(
                ChatCompletionRequestAssistantMessageArgs::default()
                    .content(response.content.clone().unwrap_or_default())
                    .tool_calls(openai_tool_calls)
                    .build()
                    .map_err(|e| format!("构建 assistant tool_calls message 失败: {}", e))?
                    .into(),
            );

            // 执行每个工具调用
            for tc in tool_calls {
                let tool_result = execute_tool_call(engine, &tc.name, &tc.arguments)
                    .unwrap_or_else(|e| format!("错误: {}", e));

                messages.push(
                    ChatCompletionRequestToolMessageArgs::default()
                        .tool_call_id(tc.id.clone())
                        .content(tool_result)
                        .build()
                        .map_err(|e| format!("构建 tool message 失败: {}", e))?
                        .into(),
                );
            }

            if round == self.max_rounds - 1 {
                final_content = "已完成操作（达到最大轮次限制）".into();
            }
        }

        let canvas_state = engine.canvas.lock().unwrap().clone();

        Ok(SchedulerResult {
            message: final_content,
            canvas_state,
        })
    }
}

pub struct SchedulerResult {
    pub message: String,
    pub canvas_state: Option<crate::engine::canvas_state::CanvasState>,
}

/// 执行单个工具调用，返回 JSON 字符串结果
fn execute_tool_call(
    engine: &AppEngine,
    name: &str,
    arguments: &str,
) -> Result<String, String> {
    log::info!("执行工具: {} args={}", name, arguments);

    let args: Value = serde_json::from_str(arguments)
        .map_err(|e| {
            log::error!("工具参数解析失败: {} | raw={}", e, arguments);
            format!("参数解析失败: {}", e)
        })?;

    let mut canvas = engine.canvas.lock().unwrap();
    let state = canvas.as_mut().ok_or("画布未初始化")?;

    match name {
        "add_node" => {
            let node_type = crate::engine::canvas_state::NodeType::from_str(
                args["type"].as_str().unwrap_or("process"),
            )
            .map_err(|e| format!("{}", e))?;
            let label = args["label"].as_str().unwrap_or("未命名").to_string();
            let node = crate::engine::node_ops::add_node(
                &mut state.nodes,
                node_type,
                label,
                None,
                None,
            );
            Ok(serde_json::json!({"node_id": node.id, "label": node.label}).to_string())
        }
        "add_nodes_batch" => {
            let nodes = args["nodes"]
                .as_array()
                .ok_or("nodes 必须是数组")?;
            let batch: Vec<_> = nodes
                .iter()
                .map(|n| {
                    let nt = crate::engine::canvas_state::NodeType::from_str(
                        n["type"].as_str().unwrap_or("process"),
                    )
                    .unwrap_or(crate::engine::canvas_state::NodeType::Process);
                    let label = n["label"].as_str().unwrap_or("未命名").to_string();
                    (nt, label)
                })
                .collect();
            let created =
                crate::engine::node_ops::add_nodes_batch(&mut state.nodes, batch);
            // 自动布局
            let moved = crate::engine::layout::auto_layout(
                &mut state.nodes,
                &state.edges,
                crate::engine::layout::LayoutDirection::TopDown,
            );
            Ok(serde_json::json!({
                "added_count": created.len(),
                "layout_moved": moved,
                "nodes": created.iter().map(|n| {
                    serde_json::json!({"id": n.id, "type": format!("{:?}", n.node_type), "label": n.label})
                }).collect::<Vec<_>>()
            })
            .to_string())
        }
        "add_edge" => {
            let from = args["from"].as_str().ok_or("缺少 from")?;
            let to = args["to"].as_str().ok_or("缺少 to")?;
            let label = args["label"].as_str().map(|s| s.to_string());
            let edge = crate::engine::edge_ops::add_edge(
                &mut state.edges,
                from,
                to,
                label,
                None,
            )
            .map_err(|e| format!("{}", e))?;
            Ok(serde_json::json!({"edge_id": edge.id}).to_string())
        }
        "add_edges_batch" => {
            let edges = args["edges"]
                .as_array()
                .ok_or("edges 必须是数组")?;
            let batch: Vec<_> = edges
                .iter()
                .map(|e| crate::engine::edge_ops::EdgeDef {
                    from: e["from"].as_str().unwrap_or("").to_string(),
                    to: e["to"].as_str().unwrap_or("").to_string(),
                    label: e["label"].as_str().map(|s| s.to_string()),
                    style: None,
                })
                .collect();
            let created =
                crate::engine::edge_ops::add_edges_batch(&mut state.edges, batch);
            Ok(serde_json::json!({"count": created.len()}).to_string())
        }
        "update_node" => {
            let node_id = args["node_id"].as_str().ok_or("缺少 node_id")?;
            let label = args["label"].as_str().map(|s| s.to_string());
            let fill = args["fill"].as_str().map(|s| s.to_string());
            let stroke = args["stroke"].as_str().map(|s| s.to_string());
            // 更新样式
            if let Some(f) = fill {
                crate::engine::style_ops::set_element_style(
                    &mut state.nodes,
                    node_id,
                    Some(f),
                    stroke,
                    None,
                )
                .map_err(|e| format!("{}", e))?;
            }
            if let Some(l) = label {
                crate::engine::node_ops::update_node(
                    &mut state.nodes,
                    node_id,
                    Some(l),
                    None,
                    None,
                )
                .map_err(|e| format!("{}", e))?;
            }
            Ok(serde_json::json!({"success": true}).to_string())
        }
        "delete_node" => {
            let node_id = args["node_id"].as_str().ok_or("缺少 node_id")?;
            crate::engine::node_ops::delete_node(
                &mut state.nodes,
                &mut state.edges,
                node_id,
            )
            .map_err(|e| format!("{}", e))?;
            Ok(serde_json::json!({"success": true}).to_string())
        }
        "delete_edge" => {
            let edge_id = args["edge_id"].as_str().ok_or("缺少 edge_id")?;
            crate::engine::edge_ops::delete_edge(&mut state.edges, edge_id)
                .map_err(|e| format!("{}", e))?;
            Ok(serde_json::json!({"success": true}).to_string())
        }
        "auto_layout" => {
            let dir_str = args["direction"].as_str().unwrap_or("top_down");
            let direction = match dir_str {
                "left_right" => crate::engine::layout::LayoutDirection::LeftRight,
                _ => crate::engine::layout::LayoutDirection::TopDown,
            };
            let moved = crate::engine::layout::auto_layout(
                &mut state.nodes,
                &state.edges,
                direction,
            );
            Ok(serde_json::json!({"moved_count": moved}).to_string())
        }
        "set_theme" => {
            let theme_str = args["theme"].as_str().unwrap_or("default");
            let theme = crate::engine::canvas_state::Theme::from_str(theme_str)
                .map_err(|e| format!("{}", e))?;
            crate::engine::style_ops::apply_theme(&mut state.nodes, &theme);
            state.theme = theme;
            Ok(serde_json::json!({"success": true}).to_string())
        }
        "get_canvas_state" => Ok(
            serde_json::to_string(&*state).unwrap_or_else(|_| "{}".into()),
        ),
        _ => {
            log::error!("未知工具调用: {}", name);
            Err(format!("未知工具: {}", name))
        }
    }
}
