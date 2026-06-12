/// 中文语音指令去噪
/// 移除口语填充词、重复词，规范化标点
pub fn denoise(text: &str) -> String {
    // 填充词列表
    let fillers = [
        "嗯", "啊", "呃", "那个", "这个", "就是", "就是说",
        "然后", "那么", "的话", "吧", "嘛", "呗",
    ];

    let mut result = text.to_string();

    // 移除填充词
    for filler in &fillers {
        result = result.replace(filler, "");
    }

    // 移除多余空白
    let words: Vec<&str> = result.split_whitespace().collect();
    let mut deduped = Vec::new();
    for w in words {
        if deduped.last() != Some(&w) || w.chars().count() > 2 {
            deduped.push(w);
        }
    }

    deduped.join(" ")
}

/// 中文同音词纠错映射（常见 STT 错误）
pub fn correct_homophones(text: &str) -> String {
    let corrections = [
        ("话圆", "画圆"),
        ("话矩形", "画矩形"),
        ("举行", "矩形"),
        ("箭偷", "箭头"),
        ("借点", "节点"),
        ("亮解", "连接"),
        ("布橘", "布局"),
        ("扯消", "撤销"),
        ("中坐", "重做"),
        ("报存", "保存"),
        ("打倒", "导出"),
        ("放打", "放大"),
        ("缩笑", "缩小"),
        ("留成图", "流程图"),
        ("司维导图", "思维导图"),
        ("架构徒", "架构图"),
        // 可继续扩展
    ];

    let mut result = text.to_string();
    for (wrong, correct) in &corrections {
        if result.contains(wrong) {
            result = result.replace(wrong, correct);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_denoise() {
        let input = "嗯那个画一个就是红色的矩形";
        let output = denoise(input);
        assert!(!output.contains("嗯"));
        assert!(!output.contains("那个"));
        assert!(!output.contains("就是"));
        assert!(output.contains("红色的矩形"));
    }

    #[test]
    fn test_homophone() {
        let input = "画一个举行然后话圆";
        let output = correct_homophones(input);
        assert!(output.contains("矩形"));
        assert!(output.contains("画圆"));
        // "然后" 不在映射表中，应保留
        assert!(output.contains("然后"));
    }
}
