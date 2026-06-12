// TODO: Task 4.3 — 本地快捷指令匹配

/// 快捷指令匹配结果
pub struct QuickAction {
    pub name: String,
    pub params: serde_json::Value,
}

/// 尝试匹配本地快捷指令，匹配失败返回 None
pub fn try_match(text: &str) -> Option<QuickAction> {
    // 暂不实现，全部走 LLM
    let _ = text;
    None
}
