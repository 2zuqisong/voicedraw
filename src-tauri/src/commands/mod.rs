use crate::engine::AppEngine;
use crate::preprocessor::{self, PreprocessResult};
use crate::llm;
use tauri::Emitter;

// 全局 engine 实例
static ENGINE: std::sync::LazyLock<AppEngine> =
    std::sync::LazyLock::new(|| AppEngine::new());

#[tauri::command]
pub async fn process_command(
    text: String,
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let result = preprocessor::preprocess(&text);

    match result {
        PreprocessResult::LocalAction { action, params } => {
            execute_quick_action(&action, &params, &app)
        }
        PreprocessResult::NeedsLLM { cleaned_text } => {
            // 从环境变量读取 API Key（后续 Phase 7 改为配置）
            let api_key = std::env::var("DEEPSEEK_API_KEY")
                .unwrap_or_else(|_| "sk-placeholder".into());

            let scheduler = llm::scheduler::LLMScheduler::new(api_key);
            let history: Vec<(String, String)> = vec![]; // TODO: Phase 6 接入对话历史

            match scheduler.process(&cleaned_text, &history, &ENGINE).await {
                Ok(result) => {
                    // 保存快照用于 undo/redo
                    if let Some(ref state) = result.canvas_state {
                        ENGINE.snapshots.lock().unwrap().save(state.clone());
                        let _ = app.emit("canvas-updated", state);
                    }
                    Ok(serde_json::json!({
                        "success": true,
                        "message": result.message,
                        "canvas_state": result.canvas_state
                    }))
                }
                Err(e) => {
                    Ok(serde_json::json!({
                        "success": false,
                        "message": format!("LLM 处理失败: {}", e),
                        "canvas_state": null
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
    execute_quick_action(&action, &serde_json::Value::Null, &app)
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
        _ => return Err(format!("未知快捷操作: {}", action)),
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
