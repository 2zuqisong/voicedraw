pub mod denoise;
pub mod quick_match;
pub mod context_enrich;

/// 预处理结果
pub enum PreprocessResult {
    /// 需要送 LLM 处理
    NeedsLLM { cleaned_text: String },
    /// 本地快捷指令匹配成功，不需 LLM
    LocalAction { action: String, params: serde_json::Value },
}

/// 预处理管道
pub fn preprocess(text: &str) -> PreprocessResult {
    let cleaned = denoise::denoise(text);
    let corrected = denoise::correct_homophones(&cleaned);

    if corrected.trim().is_empty() {
        return PreprocessResult::NeedsLLM { cleaned_text: String::new() };
    }

    // 尝试快捷匹配
    if let Some(action) = quick_match::try_match(&corrected) {
        return PreprocessResult::LocalAction { action: action.name, params: action.params };
    }

    PreprocessResult::NeedsLLM { cleaned_text: corrected }
}
