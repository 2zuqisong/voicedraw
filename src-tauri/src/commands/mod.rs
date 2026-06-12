// 后续 Task 将完善这两个函数，届时会使用 CanvasState 等类型
#[tauri::command]
pub async fn process_command(text: String) -> Result<serde_json::Value, String> {
    // 暂时返回模拟数据
    Ok(serde_json::json!({
        "success": true,
        "message": format!("收到指令: {}", text),
        "canvas_state": null
    }))
}

#[tauri::command]
pub async fn quick_action(action: String) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "success": true,
        "message": format!("快捷操作: {}", action),
        "canvas_state": null
    }))
}
