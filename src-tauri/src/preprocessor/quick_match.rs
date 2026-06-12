use serde_json::Value;

pub struct QuickAction {
    pub name: String,
    pub params: Value,
}

/// 本地快捷指令匹配表
/// 复杂度 O(n)，因指令量极少（~10 条），无需优化
pub fn try_match(text: &str) -> Option<QuickAction> {
    let t = text.trim();

    // 精确匹配
    let patterns: &[(&[&str], &str, Value)] = &[
        (&["撤销", "回退", "上一步", "取消上一步"], "undo", Value::Null),
        (&["重做", "恢复", "下一步"], "redo", Value::Null),
        (&["清空画布", "全部删除", "清除全部", "清空"], "clear_canvas", Value::Null),
        (&["放大", "放大一点"], "zoom_in", Value::Null),
        (&["缩小", "缩小一点"], "zoom_out", Value::Null),
        (&["适应窗口", "适合屏幕", "全部显示", "显示全部"], "fit_to_screen", Value::Null),
        (&["导出", "保存", "导出图片", "保存图片"], "export", serde_json::json!({"format": "png"})),
        (&["导出PNG", "导出png", "保存PNG"], "export", serde_json::json!({"format": "png"})),
        (&["导出SVG", "导出svg", "保存SVG"], "export", serde_json::json!({"format": "svg"})),
    ];

    for (keywords, action_name, params) in patterns {
        for kw in *keywords {
            if t.contains(kw) && t.chars().count() <= kw.chars().count() + 5 {
                return Some(QuickAction {
                    name: action_name.to_string(),
                    params: params.clone(),
                });
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_undo_match() {
        let action = try_match("撤销").unwrap();
        assert_eq!(action.name, "undo");
    }

    #[test]
    fn test_zoom_in() {
        let action = try_match("放大").unwrap();
        assert_eq!(action.name, "zoom_in");
    }

    #[test]
    fn test_export_png() {
        let action = try_match("导出PNG").unwrap();
        assert_eq!(action.name, "export");
        assert_eq!(action.params["format"], "png");
    }

    #[test]
    fn test_complex_not_matched() {
        // 复杂指令不应该被快捷匹配拦截
        assert!(try_match("画一个用户登录流程图").is_none());
    }
}
