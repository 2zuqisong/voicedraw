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
    log::info!("预处理开始: '{}'", text);

    let cleaned = denoise::denoise(text);
    let corrected = denoise::correct_homophones(&cleaned);

    log::info!("预处理: 去噪后='{}', 纠错后='{}'", cleaned, corrected);

    if corrected.trim().is_empty() {
        log::warn!("预处理后文本为空");
        return PreprocessResult::NeedsLLM {
            cleaned_text: String::new(),
        };
    }

    // 尝试快捷匹配
    if let Some(action) = quick_match::try_match(&corrected) {
        log::info!("快捷匹配成功: {}", action.name);
        return PreprocessResult::LocalAction {
            action: action.name,
            params: action.params,
        };
    }

    log::info!("未匹配快捷指令，转 LLM 处理");
    PreprocessResult::NeedsLLM {
        cleaned_text: corrected,
    }
}
