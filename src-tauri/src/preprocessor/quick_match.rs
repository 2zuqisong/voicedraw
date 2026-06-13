use pinyin::ToPinyin;
use serde_json::Value;

pub struct QuickAction {
    pub name: String,
    pub params: Value,
}

/// 快捷指令条目：关键词列表 + pinyin + 动作
struct Pattern {
    keywords: &'static [&'static str],
    pinyin: &'static str, // 关键词拼音（用于 STT 中文输入的第二道防线）
    action: &'static str,
    params: Value,
}

/// 本地快捷指令匹配表
/// 两阶段：精确匹配 → 拼音模糊回退
pub fn try_match(text: &str) -> Option<QuickAction> {
    let t = text.trim();
    let char_count = t.chars().count();

    let patterns: &[Pattern] = &[
        Pattern {
            keywords: &["撤销", "回退", "上一步", "取消上一步"],
            pinyin: "chexiao",
            action: "undo",
            params: Value::Null,
        },
        Pattern {
            keywords: &["重做", "恢复", "下一步"],
            pinyin: "zhongzuo",
            action: "redo",
            params: Value::Null,
        },
        Pattern {
            keywords: &["清空画布", "全部删除", "清除全部", "清空"],
            pinyin: "qingkonghuabu",
            action: "clear_canvas",
            params: Value::Null,
        },
        Pattern {
            keywords: &["放大", "放大一点"],
            pinyin: "fangda",
            action: "zoom_in",
            params: Value::Null,
        },
        Pattern {
            keywords: &["缩小", "缩小一点"],
            pinyin: "suoxiao",
            action: "zoom_out",
            params: Value::Null,
        },
        Pattern {
            keywords: &["适应窗口", "适合屏幕", "全部显示", "显示全部"],
            pinyin: "shiyingchuangkou",
            action: "fit_to_screen",
            params: Value::Null,
        },
        Pattern {
            keywords: &["导出", "保存", "导出图片", "保存图片"],
            pinyin: "daochu",
            action: "export",
            params: serde_json::json!({"format": "png"}),
        },
        Pattern {
            keywords: &["导出PNG", "导出png", "保存PNG"],
            pinyin: "daochuPNG",
            action: "export",
            params: serde_json::json!({"format": "png"}),
        },
        Pattern {
            keywords: &["导出SVG", "导出svg", "保存SVG"],
            pinyin: "daochuSVG",
            action: "export",
            params: serde_json::json!({"format": "svg"}),
        },
    ];

    // 第一道：文本精确匹配（放宽长度限制至 +10 字符）
    for p in patterns {
        for kw in p.keywords {
            if t.contains(kw) && char_count <= kw.chars().count() + 10 {
                return Some(QuickAction {
                    name: p.action.to_string(),
                    params: p.params.clone(),
                });
            }
        }
    }

    // 第二道：拼音匹配（仅对短输入有效，避免复杂指令误匹配）
    if char_count > 10 {
        return None;
    }

    let input_pinyin = to_pinyin_flat(t);
    if input_pinyin.len() < 4 {
        return None; // 太短无法有效匹配
    }

    let mut best: Option<(&Pattern, usize)> = None; // (pattern, distance)
    for p in patterns {
        let dist = levenshtein_distance(&input_pinyin, p.pinyin);
        let threshold = (p.pinyin.chars().count() / 3).max(2); // 允许约 1/3 误差
        if dist <= threshold {
            match best {
                None => best = Some((p, dist)),
                Some((_, prev_dist)) if dist < prev_dist => best = Some((p, dist)),
                _ => {}
            }
        }
    }

    best.map(|(p, _)| QuickAction {
        name: p.action.to_string(),
        params: p.params.clone(),
    })
}

/// 将中文文本转为无空格拼音字符串（无声调）
fn to_pinyin_flat(text: &str) -> String {
    text.chars()
        .filter_map(|c| c.to_pinyin())
        .map(|p| p.plain().to_string())
        .collect::<Vec<_>>()
        .join("")
}

/// Levenshtein 编辑距离
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let m = a_chars.len();
    let n = b_chars.len();

    let mut dp = vec![vec![0usize; n + 1]; m + 1];
    for i in 0..=m {
        dp[i][0] = i;
    }
    for j in 0..=n {
        dp[0][j] = j;
    }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            dp[i][j] = (dp[i - 1][j] + 1) // deletion
                .min(dp[i][j - 1] + 1) // insertion
                .min(dp[i - 1][j - 1] + cost); // substitution
        }
    }
    dp[m][n]
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

    #[test]
    fn test_fuzzy_length_tolerance() {
        // 放宽长度限制后，"放大一点看看" 应能匹配
        let action = try_match("放大一点看看").unwrap();
        assert_eq!(action.name, "zoom_in");
    }

    #[test]
    fn test_pinyin_match_chexiao() {
        // 拼音 "chexiao" 应匹配撤销
        let action = try_match("扯消").unwrap();
        assert_eq!(action.name, "undo");
    }

    #[test]
    fn test_pinyin_match_fangda() {
        let action = try_match("放打").unwrap();
        assert_eq!(action.name, "zoom_in");
    }

    #[test]
    fn test_pinyin_no_false_positive() {
        // 复杂指令不应被拼音匹配误捕获
        assert!(try_match("画一个用户登录流程图包含验证").is_none());
        assert!(try_match("画微服务架构图共六个组件").is_none());
    }

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein_distance("chexiao", "chexiao"), 0);
        assert_eq!(levenshtein_distance("chexiao", "chaxiao"), 1);
        assert_eq!(levenshtein_distance("fangda", "fangda"), 0);
    }
}
