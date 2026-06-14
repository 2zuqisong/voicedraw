use crate::engine::AppEngine;
use crate::preprocessor::{self, PreprocessResult};
use crate::llm;
use std::sync::Mutex;
use tauri::Emitter;

// 全局 engine 实例
static ENGINE: std::sync::LazyLock<AppEngine> =
    std::sync::LazyLock::new(|| AppEngine::new());

/// 全局 LLM 调度器（用于跨请求的 plan 缓存）
static LLM_SCHEDULER: std::sync::LazyLock<Mutex<Option<llm::scheduler::LLMScheduler>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));

#[tauri::command]
pub async fn process_command(
    text: String,
    app: tauri::AppHandle,
    // 可选：前端传入的 DeepSeek API Key（优先于环境变量）
    deepseek_key: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!("process_command: '{}'", text);

    let result = preprocessor::preprocess(&text);

    match result {
        PreprocessResult::LocalAction { action, params } => {
            log::info!("快捷指令匹配: action={}", action);
            execute_quick_action(&action, &params, &app)
        }
        PreprocessResult::NeedsLLM { cleaned_text } => {
            log::info!("需要 LLM 处理: '{}'", cleaned_text);
            // 优先使用前端传入的 key，其次用环境变量
            let api_key = deepseek_key
                .filter(|k| !k.is_empty())
                .or_else(|| std::env::var("DEEPSEEK_API_KEY").ok())
                .unwrap_or_else(|| "sk-placeholder".into());

            // 用对话上下文丰富用户文本（代词消解）
            let enriched_text = {
                let ctx = ENGINE.context.lock().unwrap();
                let canvas = ENGINE.canvas.lock().unwrap();
                let nodes = canvas.as_ref().map(|c| &c.nodes);
                match nodes {
                    Some(n) => ctx.enrich_user_text(&cleaned_text, n),
                    None => cleaned_text.clone(),
                }
            };

            // 从对话上下文中获取历史
            let history = {
                let ctx = ENGINE.context.lock().unwrap();
                ctx.to_history()
            };

            // take() 模式：取出 scheduler 后释放锁，避免 MutexGuard 跨越 await
            let mut scheduler = {
                let mut guard = LLM_SCHEDULER.lock().unwrap();
                if guard.is_none() {
                    *guard = Some(llm::scheduler::LLMScheduler::new(api_key));
                }
                guard.take().unwrap()
            };

            let result = scheduler.process(&enriched_text, &history, &ENGINE).await;

            // 将 scheduler 放回全局
            *LLM_SCHEDULER.lock().unwrap() = Some(scheduler);

            match result {
                Ok(llm::scheduler::ProcessResult::Executed(result)) => {
                    log::info!("LLM 处理成功: {}", result.message);

                    // 保存快照用于 undo/redo
                    if let Some(ref state) = result.canvas_state {
                        ENGINE.snapshots.lock().unwrap().save(state.clone());
                        let _ = app.emit("canvas-updated", state);
                    }

                    // 记录本轮对话
                    {
                        let mut ctx = ENGINE.context.lock().unwrap();
                        ctx.add_turn(
                            cleaned_text.clone(),
                            result.message.clone(),
                            vec![],
                        );
                    }

                    Ok(serde_json::json!({
                        "success": true,
                        "message": result.message,
                        "canvas_state": result.canvas_state,
                        "pending_plan": null,
                        "pending_action": result.pending_action
                    }))
                }
                Ok(llm::scheduler::ProcessResult::PendingPlan { plan_json, message }) => {
                    log::info!("LLM 返回待确认计划");
                    Ok(serde_json::json!({
                        "success": true,
                        "message": message,
                        "canvas_state": null,
                        "pending_plan": serde_json::from_str::<serde_json::Value>(&plan_json)
                            .unwrap_or(serde_json::Value::Null)
                    }))
                }
                Err(e) => {
                    log::error!("LLM 处理失败: {}", e);
                    Ok(serde_json::json!({
                        "success": false,
                        "message": format!("LLM 处理失败: {}", e),
                        "canvas_state": null,
                        "pending_plan": null
                    }))
                }
            }
        }
    }
}

#[tauri::command]
pub async fn quick_action(
    action: String,
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    log::info!("quick_action: {}", action);
    execute_quick_action(&action, &serde_json::Value::Null, &app)
}

#[tauri::command]
pub async fn confirm_plan(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    log::info!("confirm_plan: 用户确认执行");

    let history = {
        let ctx = ENGINE.context.lock().unwrap();
        ctx.to_history()
    };

    // take() 模式：取出 scheduler，释放锁，await 后放回
    let mut scheduler = LLM_SCHEDULER
        .lock()
        .unwrap()
        .take()
        .ok_or("调度器未初始化")?;

    // 在 await 前捕获用户文本（confirm_plan 内部会 take 清空）
    let plan_user_text = scheduler.cached_user_text.clone().unwrap_or_default();

    let result = scheduler.confirm_plan(&history, &ENGINE).await;

    // 放回
    *LLM_SCHEDULER.lock().unwrap() = Some(scheduler);

    match result {
        Ok(result) => {
            log::info!("计划执行成功: {}", result.message);
            if let Some(ref state) = result.canvas_state {
                ENGINE.snapshots.lock().unwrap().save(state.clone());
                let _ = app.emit("canvas-updated", state);
            }
            {
                let mut ctx = ENGINE.context.lock().unwrap();
                ctx.add_turn(plan_user_text, result.message.clone(), vec![]);
            }
            Ok(serde_json::json!({
                "success": true,
                "message": result.message,
                "canvas_state": result.canvas_state,
                "pending_action": result.pending_action
            }))
        }
        Err(e) => {
            log::error!("计划执行失败: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "message": format!("执行失败: {}", e),
                "canvas_state": null
            }))
        }
    }
}

#[tauri::command]
pub async fn cancel_plan() -> Result<serde_json::Value, String> {
    log::info!("cancel_plan: 用户取消计划");
    let mut guard = LLM_SCHEDULER.lock().unwrap();
    if let Some(ref mut scheduler) = *guard {
        scheduler.cancel_plan();
    }
    Ok(serde_json::json!({"success": true, "message": "已取消"}))
}

#[tauri::command]
pub async fn modify_plan(new_text: String) -> Result<serde_json::Value, String> {
    log::info!("modify_plan: 用户修改指令 '{}'", new_text);

    let history = {
        let ctx = ENGINE.context.lock().unwrap();
        ctx.to_history()
    };

    // take() 模式
    let mut scheduler = LLM_SCHEDULER
        .lock()
        .unwrap()
        .take()
        .ok_or("调度器未初始化")?;

    scheduler.modify_plan();
    let result = scheduler.process(&new_text, &history, &ENGINE).await;

    // 放回
    *LLM_SCHEDULER.lock().unwrap() = Some(scheduler);

    match result {
        Ok(llm::scheduler::ProcessResult::Executed(result)) => {
            Ok(serde_json::json!({
                "success": true,
                "message": result.message,
                "canvas_state": result.canvas_state,
                "pending_plan": null
            }))
        }
        Ok(llm::scheduler::ProcessResult::PendingPlan { plan_json, message }) => {
            Ok(serde_json::json!({
                "success": true,
                "message": message,
                "canvas_state": null,
                "pending_plan": serde_json::from_str::<serde_json::Value>(&plan_json)
                    .unwrap_or(serde_json::Value::Null)
            }))
        }
        Err(e) => {
            Ok(serde_json::json!({
                "success": false,
                "message": format!("处理失败: {}", e),
                "canvas_state": null,
                "pending_plan": null
            }))
        }
    }
}

/// 前端调用：将捕获的 canvas 图像发送到 DashScope 进行风格转换
#[tauri::command]
pub async fn apply_style_transfer(
    image_base64: String,
    prompt: String,
    node_ids: Vec<String>,
    // 可选：前端传入的 DashScope API Key（优先于环境变量）
    dashscope_key: Option<String>,
) -> Result<serde_json::Value, String> {
    log::info!(
        "apply_style_transfer: prompt='{}', node_ids={:?}, image_len={}",
        prompt,
        node_ids,
        image_base64.len()
    );

    let api_key = dashscope_key
        .filter(|k| !k.is_empty())
        .or_else(|| std::env::var("DASHSCOPE_API_KEY").ok())
        .ok_or_else(|| "未设置通义万相 API Key（请在设置面板填写或设置 DASHSCOPE_API_KEY 环境变量）".to_string())?;

    let result_image = crate::engine::style_transfer::apply_style_transfer(
        &api_key,
        &image_base64,
        &prompt,
    )
    .await
    .map_err(|e| {
        log::error!("风格转换失败: {}", e);
        format!("风格转换失败: {}", e)
    })?;

    Ok(serde_json::json!({
        "success": true,
        "image_base64": result_image,
        "replaced_node_ids": node_ids
    }))
}

fn execute_quick_action(
    action: &str,
    _params: &serde_json::Value,
    app: &tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let mut canvas = ENGINE.canvas.lock().unwrap();
    let canvas_state = canvas.as_mut().ok_or("画布未初始化")?;

    let message = match action {
        "undo" => {
            let current = canvas_state.clone();
            let mut snapshots = ENGINE.snapshots.lock().unwrap();
            if let Some(prev) = snapshots.undo(current) {
                *canvas_state = prev;
                "已撤销".to_string()
            } else {
                "没有可撤销的操作".to_string()
            }
        }
        "redo" => {
            let current = canvas_state.clone();
            let mut snapshots = ENGINE.snapshots.lock().unwrap();
            if let Some(next) = snapshots.redo(current) {
                *canvas_state = next;
                "已重做".to_string()
            } else {
                "没有可重做的操作".to_string()
            }
        }
        "clear_canvas" => {
            canvas_state.nodes.clear();
            canvas_state.edges.clear();
            "画布已清空".to_string()
        }
        "zoom_in" | "zoom_out" | "fit_to_screen" | "export" => {
            // 缩放和导出由前端处理，这里返回信号即可
            format!("快捷操作: {}", action)
        }
        _ => {
            log::warn!("未知快捷操作: {}", action);
            return Err(format!("未知快捷操作: {}", action));
        }
    };

    // 发送事件通知前端更新
    let state = canvas_state.clone();
    let _ = app.emit("canvas-updated", &state);

    Ok(serde_json::json!({
        "success": true,
        "message": message,
        "canvas_state": state
    }))
}
