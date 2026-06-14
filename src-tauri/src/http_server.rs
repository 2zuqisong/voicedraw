//! HTTP API server — mirrors Tauri commands for browser access.
//! Runs on port 1421 alongside the Tauri desktop app.

use axum::{
    Router, Json, routing::{get, post},
    http::{Method, header::CONTENT_TYPE},
};
use tower_http::cors::{CorsLayer, Any};
use serde::Deserialize;

use crate::commands::{ENGINE, LLM_SCHEDULER};
use crate::preprocessor::{self, PreprocessResult};
use crate::llm;

// ── Request types (camelCase from JS frontend) ───────────────────────

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProcessCommandArgs {
    text: String,
    llm_api_key: Option<String>,
    llm_endpoint: Option<String>,
    llm_model: Option<String>,
    canvas_mode: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QuickActionArgs {
    action: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ModifyPlanArgs {
    new_text: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct StyleTransferArgs {
    image_base64: String,
    prompt: String,
    node_ids: Vec<String>,
    dashscope_key: Option<String>,
}

// ── Handlers ─────────────────────────────────────────────────────────

async fn handle_process_command(
    Json(args): Json<ProcessCommandArgs>,
) -> Json<serde_json::Value> {
    log::info!("[http] process_command: '{}'", args.text);

    let result = preprocessor::preprocess(&args.text);

    match result {
        PreprocessResult::LocalAction { action, params } => {
            log::info!("[http] 快捷指令匹配: action={}", action);
            Json(http_quick_action_inner(&action, &params))
        }
        PreprocessResult::NeedsLLM { cleaned_text } => {
            log::info!("[http] 需要 LLM 处理: '{}'", cleaned_text);

            let api_key = args.llm_api_key
                .filter(|k| !k.is_empty())
                .or_else(|| std::env::var("DEEPSEEK_API_KEY").ok())
                .unwrap_or_else(|| "sk-placeholder".into());
            let endpoint = args.llm_endpoint
                .filter(|e| !e.is_empty())
                .unwrap_or_else(|| "https://api.deepseek.com".into());
            let model = args.llm_model
                .filter(|m| !m.is_empty())
                .unwrap_or_else(|| "deepseek-chat".into());

            // Enrich text with conversation context
            let enriched_text = {
                let ctx = ENGINE.context.lock().unwrap();
                let canvas = ENGINE.canvas.lock().unwrap();
                let nodes = canvas.as_ref().map(|c| &c.nodes);
                match nodes {
                    Some(n) => ctx.enrich_user_text(&cleaned_text, n),
                    None => cleaned_text.clone(),
                }
            };

            let history = {
                let ctx = ENGINE.context.lock().unwrap();
                ctx.to_history()
            };

            // take() pattern — extract scheduler, release lock, await, put back
            let mut scheduler = {
                let mut guard = LLM_SCHEDULER.lock().unwrap();
                if guard.is_none() {
                    *guard = Some(llm::scheduler::LLMScheduler::new(
                        api_key, endpoint, model,
                    ));
                }
                guard.take().unwrap()
            };

            let result = scheduler.process(&enriched_text, &history, &ENGINE, args.canvas_mode.as_deref()).await;

            // Put scheduler back
            *LLM_SCHEDULER.lock().unwrap() = Some(scheduler);

            match result {
                Ok(llm::scheduler::ProcessResult::Executed(result)) => {
                    log::info!("[http] LLM 处理成功: {}", result.message);

                    if let Some(ref state) = result.canvas_state {
                        ENGINE.snapshots.lock().unwrap().save(state.clone());
                    }

                    {
                        let mut ctx = ENGINE.context.lock().unwrap();
                        ctx.add_turn(
                            cleaned_text.clone(),
                            result.message.clone(),
                            vec![],
                        );
                    }

                    Json(serde_json::json!({
                        "success": true,
                        "message": result.message,
                        "canvas_state": result.canvas_state,
                        "pending_plan": null,
                        "pending_action": result.pending_action
                    }))
                }
                Ok(llm::scheduler::ProcessResult::PendingPlan { plan_json, message }) => {
                    log::info!("[http] LLM 返回待确认计划");
                    Json(serde_json::json!({
                        "success": true,
                        "message": message,
                        "canvas_state": null,
                        "pending_plan": serde_json::from_str::<serde_json::Value>(&plan_json)
                            .unwrap_or(serde_json::Value::Null)
                    }))
                }
                Err(e) => {
                    log::error!("[http] LLM 处理失败: {}", e);
                    Json(serde_json::json!({
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

async fn handle_quick_action(
    Json(args): Json<QuickActionArgs>,
) -> Json<serde_json::Value> {
    log::info!("[http] quick_action: {}", args.action);
    Json(http_quick_action_inner(&args.action, &serde_json::Value::Null))
}

async fn handle_confirm_plan() -> Json<serde_json::Value> {
    log::info!("[http] confirm_plan");

    let history = {
        let ctx = ENGINE.context.lock().unwrap();
        ctx.to_history()
    };

    // take() pattern
    let mut scheduler = match LLM_SCHEDULER.lock().unwrap().take() {
        Some(s) => s,
        None => {
            return Json(serde_json::json!({
                "success": false,
                "message": "调度器未初始化",
                "canvas_state": null
            }));
        }
    };

    let plan_user_text = scheduler.cached_user_text.clone().unwrap_or_default();

    let result = scheduler.confirm_plan(&history, &ENGINE).await;

    // Put scheduler back
    *LLM_SCHEDULER.lock().unwrap() = Some(scheduler);

    match result {
        Ok(result) => {
            log::info!("[http] 计划执行成功: {}", result.message);
            if let Some(ref state) = result.canvas_state {
                ENGINE.snapshots.lock().unwrap().save(state.clone());
            }
            {
                let mut ctx = ENGINE.context.lock().unwrap();
                ctx.add_turn(plan_user_text, result.message.clone(), vec![]);
            }
            Json(serde_json::json!({
                "success": true,
                "message": result.message,
                "canvas_state": result.canvas_state,
                "pending_action": result.pending_action
            }))
        }
        Err(e) => {
            log::error!("[http] 计划执行失败: {}", e);
            Json(serde_json::json!({
                "success": false,
                "message": format!("执行失败: {}", e),
                "canvas_state": null
            }))
        }
    }
}

// ── Inner helpers (no AppHandle, used by HTTP handlers) ───────────────

fn http_quick_action_inner(action: &str, _params: &serde_json::Value) -> serde_json::Value {
    let mut canvas = ENGINE.canvas.lock().unwrap();
    let canvas_state = match canvas.as_mut() {
        Some(s) => s,
        None => {
            return serde_json::json!({
                "success": false,
                "message": "画布未初始化",
                "canvas_state": null
            });
        }
    };

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
            format!("快捷操作: {}", action)
        }
        _ => {
            log::warn!("[http] 未知快捷操作: {}", action);
            return serde_json::json!({
                "success": false,
                "message": format!("未知快捷操作: {}", action),
                "canvas_state": null
            });
        }
    };

    let state = canvas_state.clone();
    serde_json::json!({
        "success": true,
        "message": message,
        "canvas_state": state
    })
}

async fn handle_cancel_plan() -> Json<serde_json::Value> {
    log::info!("[http] cancel_plan");
    let mut guard = LLM_SCHEDULER.lock().unwrap();
    if let Some(ref mut scheduler) = *guard {
        scheduler.cancel_plan();
    }
    Json(serde_json::json!({"success": true, "message": "已取消"}))
}

async fn handle_modify_plan(
    Json(args): Json<ModifyPlanArgs>,
) -> Json<serde_json::Value> {
    log::info!("[http] modify_plan: '{}'", args.new_text);

    let history = {
        let ctx = ENGINE.context.lock().unwrap();
        ctx.to_history()
    };

    let scheduler_result = LLM_SCHEDULER
        .lock()
        .unwrap()
        .take()
        .ok_or("调度器未初始化");

    let mut scheduler = match scheduler_result {
        Ok(s) => s,
        Err(e) => {
            return Json(serde_json::json!({
                "success": false,
                "message": e,
                "canvas_state": null,
                "pending_plan": null
            }));
        }
    };

    scheduler.modify_plan();
    let result = scheduler.process(&args.new_text, &history, &ENGINE, None).await;

    *LLM_SCHEDULER.lock().unwrap() = Some(scheduler);

    match result {
        Ok(llm::scheduler::ProcessResult::Executed(result)) => {
            Json(serde_json::json!({
                "success": true,
                "message": result.message,
                "canvas_state": result.canvas_state,
                "pending_plan": null
            }))
        }
        Ok(llm::scheduler::ProcessResult::PendingPlan { plan_json, message }) => {
            Json(serde_json::json!({
                "success": true,
                "message": message,
                "canvas_state": null,
                "pending_plan": serde_json::from_str::<serde_json::Value>(&plan_json)
                    .unwrap_or(serde_json::Value::Null)
            }))
        }
        Err(e) => {
            Json(serde_json::json!({
                "success": false,
                "message": format!("处理失败: {}", e),
                "canvas_state": null,
                "pending_plan": null
            }))
        }
    }
}

async fn handle_style_transfer(
    Json(args): Json<StyleTransferArgs>,
) -> Json<serde_json::Value> {
    log::info!(
        "[http] apply_style_transfer: prompt='{}', node_ids={:?}, image_len={}",
        args.prompt, args.node_ids, args.image_base64.len()
    );

    let api_key = args.dashscope_key
        .filter(|k| !k.is_empty())
        .or_else(|| std::env::var("DASHSCOPE_API_KEY").ok());

    let api_key = match api_key {
        Some(k) => k,
        None => {
            return Json(serde_json::json!({
                "success": false,
                "message": "未设置通义万相 API Key（请在设置面板填写或设置 DASHSCOPE_API_KEY 环境变量）"
            }));
        }
    };

    match crate::engine::style_transfer::apply_style_transfer(
        &api_key,
        &args.image_base64,
        &args.prompt,
    ).await {
        Ok(result_image) => {
            Json(serde_json::json!({
                "success": true,
                "image_base64": result_image,
                "replaced_node_ids": args.node_ids
            }))
        }
        Err(e) => {
            log::error!("[http] 风格转换失败: {}", e);
            Json(serde_json::json!({
                "success": false,
                "message": format!("风格转换失败: {}", e)
            }))
        }
    }
}

async fn handle_reset_context() -> Json<serde_json::Value> {
    let mut ctx = ENGINE.context.lock().unwrap();
    ctx.clear();
    Json(serde_json::json!({"success": true}))
}

async fn handle_snapshot_status() -> Json<serde_json::Value> {
    let snapshots = ENGINE.snapshots.lock().unwrap();
    Json(serde_json::json!({
        "can_undo": snapshots.can_undo(),
        "can_redo": snapshots.can_redo(),
    }))
}

// ── Server startup ───────────────────────────────────────────────────

/// Start the HTTP API server on port 1421.
/// This runs alongside the Tauri app so the frontend can be accessed
/// from a regular browser at http://localhost:1420/
pub fn start() {
    std::thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers([CONTENT_TYPE]);

            let app = Router::new()
                .route("/api/process_command", post(handle_process_command))
                .route("/api/quick_action", post(handle_quick_action))
                .route("/api/confirm_plan", post(handle_confirm_plan))
                .route("/api/cancel_plan", post(handle_cancel_plan))
                .route("/api/modify_plan", post(handle_modify_plan))
                .route("/api/apply_style_transfer", post(handle_style_transfer))
                .route("/api/reset_context", post(handle_reset_context))
            .route("/api/get_snapshot_status", get(handle_snapshot_status))
                .layer(cors);

            let listener = tokio::net::TcpListener::bind("127.0.0.1:1421").await.unwrap();
            log::info!("🌐 HTTP API 服务器已启动: http://127.0.0.1:1421");
            axum::serve(listener, app).await.unwrap();
        });
    });
}
