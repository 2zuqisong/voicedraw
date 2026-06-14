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
use crate::engine::grid::GridConfig;
use crate::engine::AppEngine;

/// 从 LLM 响应中提取 JSON：处理 markdown 代码块包裹的情况
fn extract_json_from_response(content: &str) -> &str {
    let trimmed = content.trim();
    // 去掉 ```json ... ``` 或 ``` ... ``` 包裹
    if let Some(inner) = trimmed
        .strip_prefix("```json")
        .or_else(|| trimmed.strip_prefix("```"))
    {
        if let Some(json) = inner.strip_suffix("```") {
            return json.trim();
        }
    }
    trimmed
}

/// 操作计划（用于预览确认）
#[derive(Debug, Clone, serde::Serialize)]
pub struct OperationPlan {
    pub id: String,
    pub diagram_type: String,
    pub summary: String,
    pub nodes: Vec<PlanNode>,
    pub edges: Vec<PlanEdge>,
    pub grid_position: Option<(f64, f64)>,
    pub layout_direction: String,
    pub estimated_tool_calls: u32,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlanNode {
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlanEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

/// 复杂度判断 prompt
const COMPLEXITY_PROMPT: &str = r#"分析以下用户指令的复杂度。只返回 JSON，不要其他内容。

判断标准:
- simple: 单节点添加、修改颜色/文字、删除、查询状态、撤销/重做、缩放/导出
- complex: 创建2+个新节点、新建流程图/架构图/思维导图/ER图、批量修改

返回格式:
{
  "complexity": "simple" 或 "complex",
  "reason": "简要原因（中文，一句话）",
  "estimated_tool_calls": 数字
}

指令: "#;

/// LLM 调度器：管理 DeepSeek API 多轮对话循环
pub struct LLMScheduler {
    client: DeepSeekClient,
    max_rounds: u8,
    plan_cache: Option<OperationPlan>,
    /// 触发当前 plan 的原始用户文本（用于 confirm_plan 时恢复）
    pub(crate) cached_user_text: Option<String>,
    /// 当前画布模式（"vector" 或 "pixel"）
    canvas_mode: Option<String>,
}

impl LLMScheduler {
    pub fn new(api_key: String, base_url: String, model: String) -> Self {
        Self {
            client: DeepSeekClient::new(api_key, base_url, model),
            max_rounds: 5,
            plan_cache: None,
            cached_user_text: None,
            canvas_mode: None,
        }
    }

    /// 处理用户指令，返回最终回复和更新的 Canvas 状态
    pub async fn process(
        &mut self,
        user_text: &str,
        history: &[(String, String)], // (role, content)
        engine: &AppEngine,
        canvas_mode: Option<&str>,
    ) -> Result<ProcessResult, String> {
        if let Some(mode) = canvas_mode {
            self.canvas_mode = Some(mode.to_string());
        }
        log::info!(
            "LLM Scheduler: 处理指令 '{}', 历史 {} 轮",
            user_text,
            history.len() / 2
        );

        // 1. 复杂度判断
        let complexity = self.judge_complexity(user_text).await?;
        log::info!(
            "复杂度判断: {} (预估 {} 个工具调用)",
            complexity.0,
            complexity.2
        );

        if complexity.0 == "complex" && complexity.2 > 3 {
            // 复杂指令：生成计划节点 → 等待用户确认
            let nodes = self.generate_plan_nodes(user_text).await?;
            let plan = OperationPlan {
                id: uuid::Uuid::new_v4().to_string(),
                diagram_type: self.infer_diagram_type(user_text),
                summary: complexity.1.clone(),
                nodes,
                edges: vec![],
                grid_position: {
                    let canvas = engine.canvas.lock().unwrap();
                    canvas.as_ref().map(|c| {
                        let grid_cfg = GridConfig::default();
                        let (gx, gy) = grid_cfg.find_empty_anchor(&c.nodes);
                        (gx, gy)
                    })
                },
                layout_direction: "top_down".into(),
                estimated_tool_calls: complexity.2,
            };
            let plan_json = serde_json::to_string(&plan).unwrap_or_default();
            log::info!("生成操作计划: id={}", plan.id);
            self.plan_cache = Some(plan);
            self.cached_user_text = Some(user_text.to_string());
            return Ok(ProcessResult::PendingPlan {
                plan_json,
                message: format!("即将创建{}", complexity.1),
            });
        }

        // 简单指令：沿用现有多轮执行逻辑
        let result = self.execute_full(user_text, history, engine).await?;
        Ok(ProcessResult::Executed(result))
    }

    /// 复杂度判断：轻量 LLM 调用
    async fn judge_complexity(&self, user_text: &str) -> Result<(String, String, u32), String> {
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content("你是指令复杂度分析器。只返回JSON。")
                .build()
                .map_err(|e| format!("{}", e))?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(format!("{}{}", COMPLEXITY_PROMPT, user_text))
                .build()
                .map_err(|e| format!("{}", e))?
                .into(),
        ];

        let response = self.client.chat(messages, vec![], false).await?;
        let content = response.content.unwrap_or_default();
        let json_text = extract_json_from_response(&content);

        // 解析 JSON 响应
        let v: serde_json::Value = serde_json::from_str(json_text)
            .map_err(|e| format!("复杂度判断 JSON 解析失败: {} | raw={}", e, content))?;

        Ok((
            v["complexity"].as_str().unwrap_or("simple").to_string(),
            v["reason"].as_str().unwrap_or("未知操作").to_string(),
            v["estimated_tool_calls"].as_u64().unwrap_or(1) as u32,
        ))
    }

    /// 从用户指令推断图表类型
    fn infer_diagram_type(&self, text: &str) -> String {
        if text.contains("流程") {
            return "流程图".into();
        }
        if text.contains("架构") || text.contains("系统") {
            return "架构图".into();
        }
        if text.contains("思维导图") || text.contains("脑图") {
            return "思维导图".into();
        }
        if text.contains("ER") || text.contains("实体") {
            return "ER图".into();
        }
        if text.contains("时序") {
            return "时序图".into();
        }
        if text.contains("UML") {
            return "UML图".into();
        }
        "图表".into()
    }

    /// 简单生成节点摘要（从用户文本中提取关键词作为节点标签）
    async fn generate_plan_nodes(&self, user_text: &str) -> Result<Vec<PlanNode>, String> {
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content("你是图表节点提取器。从用户描述中提取节点列表。只返回 JSON 数组。每个节点有 label（中文标签）和 type（start/end/process/decision/data/subprocess/text）。")
                .build()
                .map_err(|e| format!("{}", e))?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(format!("提取此指令的所有节点: {}", user_text))
                .build()
                .map_err(|e| format!("{}", e))?
                .into(),
        ];

        let response = self.client.chat(messages, vec![], false).await?;
        let content = response.content.unwrap_or_default();
        let json_text = extract_json_from_response(&content);
        let nodes: Vec<PlanNode> = serde_json::from_str(json_text)
            .map_err(|e| format!("节点提取 JSON 解析失败: {} | raw={}", e, content))?;
        Ok(nodes)
    }

    /// 确认计划：执行完整 LLM 工具调用循环
    pub async fn confirm_plan(
        &mut self,
        history: &[(String, String)],
        engine: &AppEngine,
    ) -> Result<SchedulerResult, String> {
        let user_text = self.cached_user_text.take().unwrap_or_default();
        self.plan_cache = None;
        self.execute_full(&user_text, history, engine).await
    }

    /// 取消计划
    pub fn cancel_plan(&mut self) {
        self.plan_cache = None;
        self.cached_user_text = None;
        log::info!("操作计划已取消");
    }

    /// 修改计划：清除缓存
    pub fn modify_plan(&mut self) {
        self.plan_cache = None;
        self.cached_user_text = None;
    }

    /// 完整执行（原有 LLM 多轮工具调用逻辑）
    async fn execute_full(
        &self,
        user_text: &str,
        history: &[(String, String)],
        engine: &AppEngine,
    ) -> Result<SchedulerResult, String> {
        let mut messages: Vec<ChatCompletionRequestMessage> = Vec::new();

        // 1. System prompt（像素模式时前置强指令，覆盖矢量示例）
        let system_prompt_raw = get_system_prompt();
        let system_prompt = match self.canvas_mode.as_deref() {
            Some("pixel") => format!(
                "## 🎨 当前模式：像素绘画\n\n你只能使用像素工具：pixel_set, pixel_fill, pixel_rect, pixel_clear。\n绝对禁止使用矢量工具（add_node, add_nodes_batch, add_edge 等）。\n即使指令看起来需要矢量图形，也要用像素格子画出来。\n\n{}",
                system_prompt_raw
            ),
            _ => system_prompt_raw,
        };
        messages.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(system_prompt)
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
                let mut s = format!(
                    "当前画布: {}, 节点数: {}, 连线数: {}, 主题: {:?}",
                    c.title,
                    c.nodes.len(),
                    c.edges.len(),
                    c.theme
                );
                if let Some(ref pixel) = c.pixel {
                    s.push_str(&format!(
                        "\n像素画布: {}×{} 网格, {} 个彩色格子",
                        pixel.cols, pixel.rows, pixel.cells.len()
                    ));
                }
                s
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
        let pending_action = engine.pending_action.lock().unwrap().take();

        Ok(SchedulerResult {
            message: final_content,
            canvas_state,
            pending_action,
        })
    }
}

/// process() 的返回类型
pub enum ProcessResult {
    Executed(SchedulerResult),
    PendingPlan {
        plan_json: String,
        message: String,
    },
}

pub struct SchedulerResult {
    pub message: String,
    pub canvas_state: Option<crate::engine::canvas_state::CanvasState>,
    /// 需要前端先处理的异步操作（如风格转换需要前端捕获 canvas 图像）
    pub pending_action: Option<crate::engine::canvas_state::PendingAction>,
}

/// 像素画布泛洪填充（BFS）
fn pixel_flood_fill(
    cells: &mut std::collections::HashMap<String, String>,
    cols: u32,
    rows: u32,
    start_row: i32,
    start_col: i32,
    target_color: &Option<String>,
    fill_color: &str,
) -> usize {
    let key = format!("{},{}", start_row, start_col);
    let current = cells.get(&key).cloned();

    // 只填充同色相连区域
    if current.as_ref() != target_color.as_ref() {
        return 0;
    }
    // 如果目标色就是填充色，无需操作
    if target_color.as_ref().map_or(false, |c| c == fill_color) {
        return 0;
    }

    let mut count = 0;
    let mut stack = vec![(start_row, start_col)];
    let mut visited = std::collections::HashSet::new();
    visited.insert(key);

    while let Some((r, c)) = stack.pop() {
        let k = format!("{},{}", r, c);
        let cell_color = cells.get(&k).cloned();

        if cell_color.as_ref() == target_color.as_ref() {
            cells.insert(k, fill_color.to_string());
            count += 1;

            for (dr, dc) in &[(-1, 0), (1, 0), (0, -1), (0, 1)] {
                let nr = r + dr;
                let nc = c + dc;
                if nr >= 0 && nr < rows as i32 && nc >= 0 && nc < cols as i32 {
                    let nk = format!("{},{}", nr, nc);
                    if !visited.contains(&nk) {
                        visited.insert(nk);
                        stack.push((nr, nc));
                    }
                }
            }
        }
    }

    count
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
            let label = args["label"].as_str().unwrap_or("未命名").to_string();

            // 如果提供了网格坐标，转换为像素位置
            let position = if let (Some(gx), Some(gy)) =
                (args["grid_x"].as_f64(), args["grid_y"].as_f64())
            {
                let grid_cfg = crate::engine::grid::GridConfig::default();
                let (px, py) = grid_cfg.grid_to_pixel(gx, gy);
                Some(crate::engine::canvas_state::Position { x: px, y: py })
            } else if let (Some(x), Some(y)) = (
                args["position"]
                    .as_object()
                    .and_then(|p| p["x"].as_f64()),
                args["position"]
                    .as_object()
                    .and_then(|p| p["y"].as_f64()),
            ) {
                Some(crate::engine::canvas_state::Position { x, y })
            } else {
                None
            };

            // 检查 shape_type（几何图形路径）
            if let Some(st) = args["shape_type"].as_str() {
                let shape_type = crate::engine::canvas_state::ShapeType::from_str(st)
                    .map_err(|e| format!("{}", e))?;

                let (sub_shapes, override_size) =
                    if crate::engine::shapes::is_composite(st) {
                        crate::engine::shapes::get_composite_shapes(
                            st,
                            args["fill"].as_str(),
                        )
                        .map(|(shapes, w, h)| (Some(shapes), Some((w, h))))
                        .unwrap_or((None, None))
                    } else {
                        (None, None)
                    };

                let node_type = crate::engine::canvas_state::NodeType::Process; // placeholder

                let node = crate::engine::node_ops::add_node(
                    &mut state.nodes, node_type, label, position, None,
                );

                if let Some(n) = state.nodes.get_mut(&node.id) {
                    n.shape_type = Some(shape_type);
                    if let Some(shapes) = sub_shapes {
                        n.sub_shapes = Some(shapes);
                        if let Some((w, h)) = override_size {
                            n.size = crate::engine::canvas_state::Size { width: w, height: h };
                        }
                    }
                    if let Some(fill) = args["fill"].as_str() {
                        n.style.fill = fill.to_string();
                    }
                }

                Ok(serde_json::json!({"node_id": node.id, "label": node.label, "shape_type": st}).to_string())
            } else {
                // 原有流程图路径
                let node_type = crate::engine::canvas_state::NodeType::from_str(
                    args["type"].as_str().unwrap_or("process"),
                )
                .map_err(|e| format!("{}", e))?;

                let node = crate::engine::node_ops::add_node(
                    &mut state.nodes,
                    node_type,
                    label,
                    position,
                    None,
                );
                Ok(serde_json::json!({"node_id": node.id, "label": node.label}).to_string())
            }
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

            // 确定锚点坐标
            let (base_px, base_py) = if let (Some(gx), Some(gy)) =
                (args["grid_x"].as_f64(), args["grid_y"].as_f64())
            {
                // 用户指定了网格坐标
                let grid_cfg = crate::engine::grid::GridConfig::default();
                grid_cfg.grid_to_pixel(gx, gy)
            } else {
                // 未指定坐标，自动找空白位置
                let grid_cfg = crate::engine::grid::GridConfig::default();
                let (auto_gx, auto_gy) = grid_cfg.find_empty_anchor(&state.nodes);
                log::info!(
                    "自动锚点: grid({}, {}), pixel({}, {})",
                    auto_gx,
                    auto_gy,
                    grid_cfg.grid_to_pixel(auto_gx, auto_gy).0,
                    grid_cfg.grid_to_pixel(auto_gx, auto_gy).1
                );
                grid_cfg.grid_to_pixel(auto_gx, auto_gy)
            };

            // 偏移所有新节点到锚点
            if let Some(first) = created.first() {
                let dx = base_px - first.position.x;
                let dy = base_py - first.position.y;
                for node in &created {
                    if let Some(n) = state.nodes.get_mut(&node.id) {
                        n.position.x += dx;
                        n.position.y += dy;
                    }
                }
            }

            // 应用 shape_type + sub_shapes（几何图形节点）
            if let Some(nodes_arr) = args["nodes"].as_array() {
                for (i, node_arg) in nodes_arr.iter().enumerate() {
                    if let Some(st) = node_arg["shape_type"].as_str() {
                        if i < created.len() {
                            let node_id = &created[i].id;
                            if let Some(n) = state.nodes.get_mut(node_id) {
                                n.shape_type = Some(
                                    crate::engine::canvas_state::ShapeType::from_str(st)
                                        .unwrap_or(crate::engine::canvas_state::ShapeType::Rectangle),
                                );
                                if crate::engine::shapes::is_composite(st) {
                                    let fill_opt = node_arg["fill"].as_str();
                                    if let Some((shapes, w, h)) =
                                        crate::engine::shapes::get_composite_shapes(st, fill_opt)
                                    {
                                        n.sub_shapes = Some(shapes);
                                        n.size = crate::engine::canvas_state::Size { width: w, height: h };
                                    }
                                }
                                if let Some(fill) = node_arg["fill"].as_str() {
                                    n.style.fill = fill.to_string();
                                }
                            }
                        }
                    }
                }
            }

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
            let routing = args["routing"].as_str().unwrap_or("straight");
            let edge_style = if routing == "orthogonal" {
                Some(crate::engine::canvas_state::EdgeStyle {
                    routing: crate::engine::canvas_state::RoutingMode::Orthogonal,
                    ..Default::default()
                })
            } else {
                None
            };
            let edge = crate::engine::edge_ops::add_edge(
                &mut state.edges,
                from,
                to,
                label,
                edge_style,
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
                .map(|e| {
                    let routing = e["routing"].as_str().unwrap_or("straight");
                    let edge_style = if routing == "orthogonal" {
                        Some(crate::engine::canvas_state::EdgeStyle {
                            routing: crate::engine::canvas_state::RoutingMode::Orthogonal,
                            ..Default::default()
                        })
                    } else {
                        None
                    };
                    crate::engine::edge_ops::EdgeDef {
                        from: e["from"].as_str().unwrap_or("").to_string(),
                        to: e["to"].as_str().unwrap_or("").to_string(),
                        label: e["label"].as_str().map(|s| s.to_string()),
                        style: edge_style,
                    }
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
            let font_size = args["font_size"].as_f64();
            let stroke_width = args["stroke_width"].as_f64();
            let border_radius = args["border_radius"].as_f64();
            let text_color = args["text_color"].as_str().map(|s| s.to_string());
            let opacity = args["opacity"].as_f64();
            // 更新样式：只要任意样式参数有值就调用
            let has_style = fill.is_some()
                || stroke.is_some()
                || font_size.is_some()
                || stroke_width.is_some()
                || border_radius.is_some()
                || text_color.is_some()
                || opacity.is_some();
            if has_style {
                crate::engine::style_ops::set_element_style(
                    &mut state.nodes,
                    node_id,
                    fill,
                    stroke,
                    font_size,
                    stroke_width,
                    border_radius,
                    text_color,
                    opacity,
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
        "apply_template" => {
            let name = args["template"].as_str().ok_or("缺少 template")?;
            let grid_x = args["grid_x"].as_f64().unwrap_or(10.0);
            let grid_y = args["grid_y"].as_f64().unwrap_or(5.0);
            let title = args["title"].as_str();
            let template = crate::engine::templates::get_template(name)
                .ok_or(format!("未知模板: {}", name))?;
            let grid_cfg = crate::engine::grid::GridConfig::default();
            let (ox, oy) = grid_cfg.grid_to_pixel(grid_x, grid_y);
            let (nodes, edges) =
                crate::engine::templates::instantiate_template(template, ox, oy, title);
            let count = nodes.len() + edges.len();
            for node in nodes {
                state.nodes.insert(node.id.clone(), node);
            }
            for edge in edges {
                state.edges.insert(edge.id.clone(), edge);
            }
            // 自动排列
            let moved = crate::engine::layout::auto_layout(
                &mut state.nodes,
                &state.edges,
                crate::engine::layout::LayoutDirection::TopDown,
            );
            Ok(serde_json::json!({
                "template": name,
                "count": count,
                "moved": moved
            })
            .to_string())
        }
        "get_canvas_state" => Ok(
            serde_json::to_string(&*state).unwrap_or_else(|_| "{}".into()),
        ),
        "get_empty_anchor" => {
            let grid_cfg = crate::engine::grid::GridConfig::default();
            let (gx, gy) = grid_cfg.find_empty_anchor(&state.nodes);
            let (px, py) = grid_cfg.grid_to_pixel(gx, gy);
            Ok(serde_json::json!({
                "grid_x": gx,
                "grid_y": gy,
                "pixel_x": px,
                "pixel_y": py,
                "message": format!("推荐锚点: 网格({}, {}), 像素({}, {})", gx, gy, px, py)
            })
            .to_string())
        }
        "apply_style" => {
            let prompt = args["prompt"].as_str().unwrap_or("艺术风格").to_string();
            let node_ids: Vec<String> = args["node_ids"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();

            let target_desc = if node_ids.is_empty() {
                "整个画布".to_string()
            } else {
                format!("{} 个节点", node_ids.len())
            };

            let mut pa = engine.pending_action.lock().unwrap();
            *pa = Some(crate::engine::canvas_state::PendingAction {
                action_type: "apply_style".into(),
                prompt,
                node_ids,
            });

            Ok(serde_json::json!({
                "success": true,
                "message": format!("风格转换指令已接收，将对 {} 应用风格", target_desc)
            })
            .to_string())
        }
        // ── 像素绘画工具 ─────────────────────────────────────────────
        "pixel_set" => {
            use crate::engine::canvas_state::PixelCanvas;
            let cells = args["cells"].as_array().ok_or("cells 必须是数组")?;
            // 自动初始化像素画布
            if state.pixel.is_none() {
                state.pixel = Some(PixelCanvas {
                    cells: std::collections::HashMap::new(),
                    cell_size: 20,
                    cols: 32,
                    rows: 32,
                });
            }
            let pixel = state.pixel.as_mut().unwrap();
            let mut set_count = 0;
            let mut erase_count = 0;
            for cell in cells {
                let row = cell["row"].as_u64().ok_or("row 必须是整数")? as u32;
                let col = cell["col"].as_u64().ok_or("col 必须是整数")? as u32;
                let key = format!("{},{}", row, col);
                if let Some(color) = cell["color"].as_str() {
                    pixel.cells.insert(key, color.to_string());
                    set_count += 1;
                } else {
                    pixel.cells.remove(&key);
                    erase_count += 1;
                }
            }
            Ok(serde_json::json!({
                "success": true,
                "set": set_count,
                "erased": erase_count
            }).to_string())
        }
        "pixel_fill" => {
            use crate::engine::canvas_state::PixelCanvas;
            let row = args["row"].as_u64().ok_or("缺少 row")? as i32;
            let col = args["col"].as_u64().ok_or("缺少 col")? as i32;
            let color = args["color"].as_str().ok_or("缺少 color")?;
            if state.pixel.is_none() {
                state.pixel = Some(PixelCanvas {
                    cells: std::collections::HashMap::new(),
                    cell_size: 20,
                    cols: 32,
                    rows: 32,
                });
            }
            let pixel = state.pixel.as_mut().unwrap();
            let key = format!("{},{}", row, col);
            let target = pixel.cells.get(&key).cloned();
            let count = pixel_flood_fill(&mut pixel.cells, pixel.cols, pixel.rows, row, col, &target, color);
            Ok(serde_json::json!({
                "success": true,
                "filled": count
            }).to_string())
        }
        "pixel_rect" => {
            use crate::engine::canvas_state::PixelCanvas;
            let row = args["row"].as_u64().ok_or("缺少 row")? as i32;
            let col = args["col"].as_u64().ok_or("缺少 col")? as i32;
            let w = args["width"].as_u64().ok_or("缺少 width")? as i32;
            let h = args["height"].as_u64().ok_or("缺少 height")? as i32;
            let color = args["color"].as_str().ok_or("缺少 color")?;
            if state.pixel.is_none() {
                state.pixel = Some(PixelCanvas {
                    cells: std::collections::HashMap::new(),
                    cell_size: 20,
                    cols: 32,
                    rows: 32,
                });
            }
            let pixel = state.pixel.as_mut().unwrap();
            let mut count = 0;
            for r in row..(row + h) {
                for c in col..(col + w) {
                    pixel.cells.insert(format!("{},{}", r, c), color.to_string());
                    count += 1;
                }
            }
            Ok(serde_json::json!({
                "success": true,
                "filled": count
            }).to_string())
        }
        "pixel_clear" => {
            if let Some(ref mut pixel) = state.pixel {
                pixel.cells.clear();
            }
            Ok(serde_json::json!({"success": true, "message": "像素画布已清空"}).to_string())
        }
        "pixel_emoji" => {
            use crate::engine::canvas_state::PixelCanvas;
            let name = args["emoji"].as_str().ok_or("缺少 emoji 名称")?;
            let row = args["row"].as_u64().unwrap_or(0) as i32;
            let col = args["col"].as_u64().unwrap_or(0) as i32;

            let (w, h, emoji_cells) = crate::engine::emoji_patterns::get_emoji(name)
                .ok_or(format!("未知表情: {}。支持: {}", name, crate::engine::emoji_patterns::emoji_names().join(", ")))?;

            if state.pixel.is_none() {
                state.pixel = Some(PixelCanvas {
                    cells: std::collections::HashMap::new(),
                    cell_size: 20,
                    cols: 32,
                    rows: 32,
                });
            }
            let pixel = state.pixel.as_mut().unwrap();
            let mut count = 0;
            for (r, c, color) in emoji_cells {
                let rr = r + row;
                let cc = c + col;
                if rr >= 0 && rr < pixel.rows as i32 && cc >= 0 && cc < pixel.cols as i32 {
                    pixel.cells.insert(format!("{},{}", rr, cc), color.to_string());
                    count += 1;
                }
            }
            Ok(serde_json::json!({
                "success": true,
                "emoji": name,
                "size": format!("{}x{}", w, h),
                "cells": count,
                "position": {"row": row, "col": col}
            }).to_string())
        }
        _ => {
            log::error!("未知工具调用: {}", name);
            Err(format!("未知工具: {}", name))
        }
    }
}
