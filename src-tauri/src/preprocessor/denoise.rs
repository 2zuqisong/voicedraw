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
    let corrections: &[(&str, &str)] = &[
        // === 图形词 (~20) ===
        ("话圆", "画圆"),
        ("话矩形", "画矩形"),
        ("话图", "画图"),
        ("举行", "矩形"),
        ("巨型", "矩形"),
        ("矩行", "矩形"),
        ("灵星", "菱形"),
        ("零星", "菱形"),
        ("山角", "三角"),
        ("拖延", "椭圆"),
        ("妥圆", "椭圆"),
        ("六边", "六边形"),
        ("箭偷", "箭头"),
        ("见偷", "箭头"),
        ("园", "圆"),
        ("方框", "矩形"),
        ("方快", "方块"),
        ("原叫矩形", "圆角矩形"),
        ("平形", "平行"),
        ("平形四边形", "平行四边形"),

        // === 颜色词 (~15) ===
        ("黄涩", "黄色"),
        ("红设", "红色"),
        ("旅色", "绿色"),
        ("蓝设", "蓝色"),
        ("层色", "橙色"),
        ("橙设", "橙色"),
        ("紫色的", "紫色的"),
        ("黑色的", "黑色的"),
        ("白色的", "白色的"),
        ("灰色的", "灰色的"),
        ("粉红色", "粉红色"),
        ("粉设", "粉色"),
        ("青色", "青色"),
        ("浅色", "浅色"),

        // === 操作词 (~15) ===
        ("扯消", "撤销"),
        ("彻消", "撤销"),
        ("中坐", "重做"),
        ("抱存", "保存"),
        ("打倒", "导出"),
        ("放打", "放大"),
        ("缩笑", "缩小"),
        ("删出", "删除"),
        ("清除", "清除"),
        ("添加", "添加"),
        ("修改", "修改"),
        ("移动", "移动"),
        ("复制", "复制"),
        ("粘贴", "粘贴"),
        ("选择", "选择"),

        // === 图类词 (~15) ===
        ("留成图", "流程图"),
        ("留程图", "流程图"),
        ("司维导图", "思维导图"),
        ("思维导徒", "思维导图"),
        ("架构徒", "架构图"),
        ("持续图", "时序图"),
        ("时序徒", "时序图"),
        ("翼啊图", "ER图"),
        ("优爱慕奥图", "UML图"),
        ("组织架构图", "组织架构图"),
        ("网络拓扑图", "网络拓扑图"),
        ("状态图", "状态图"),
        ("用力图", "用例图"),
        // ER图 和 UML图 保留原词

        // === 位置/方向词 (~10) ===
        ("昨边", "左边"),
        ("又边", "右边"),
        ("上放", "上方"),
        ("下放", "下方"),
        ("总间", "中间"),
        ("左上角", "左上角"),
        ("右上角", "右上角"),
        ("左下角", "左下角"),
        ("右下角", "右下角"),
        ("旁边", "旁边"),

        // === 连接/关系词 (~10) ===
        ("亮解", "连接"),
        ("连县", "连线"),
        ("职向", "指向"),
        ("关节", "关系"),
        ("包涵", "包含"),
        ("继承", "继承"),
        ("依赖", "依赖"),
        ("关联", "关联"),
        ("聚合", "聚合"),
        ("组合", "组合"),

        // === 数量词 (~8) ===
        ("衣阁", "一个"),
        ("良阁", "两个"),
        ("三阁", "三个"),
        ("四阁", "四个"),
        ("几阁", "几个"),
        ("衣个", "一个"),
        ("良个", "两个"),
        ("多阁", "多个"),

        // === 常见 STT 碎片 (~12) ===
        ("的化", "的话"),
        ("额那个", "那个"),
        ("在来", "再来"),
        ("还又", "还有"),
        ("在添加", "再添加"),
        ("在画", "再画"),
        ("在来一个", "再来一个"),
        ("可不可以", "可以"),
        ("能部能", "能不能"),
        ("有没有", "有没有"),
    ];

    let mut result = text.to_string();
    // 先匹配长字符串（避免 "举行" 在 "平行四边形" 之前被替换）
    let mut sorted: Vec<_> = corrections.iter().collect();
    sorted.sort_by_key(|(wrong, _)| -(wrong.chars().count() as i32));
    for (wrong, correct) in &sorted {
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

    #[test]
    fn test_homophone_graphics() {
        let cases = vec![
            ("画一个举行", "矩形"),
            ("画个灵星", "菱形"),
            ("画箭偷", "箭头"),
            ("画拖延", "椭圆"),
        ];
        for (input, expected_in_output) in cases {
            let output = correct_homophones(input);
            assert!(
                output.contains(expected_in_output),
                "input='{}' should contain '{}', got '{}'",
                input,
                expected_in_output,
                output
            );
        }
    }

    #[test]
    fn test_homophone_colors() {
        let cases = vec![
            ("改成黄涩", "黄色"),
            ("设成层色", "橙色"),
            ("用蓝设", "蓝色"),
        ];
        for (input, expected) in cases {
            assert!(correct_homophones(input).contains(expected));
        }
    }

    #[test]
    fn test_homophone_diagram_types() {
        assert!(correct_homophones("画一个留成图").contains("流程图"));
        assert!(correct_homophones("画司维导图").contains("思维导图"));
        assert!(correct_homophones("画架构徒").contains("架构图"));
    }

    #[test]
    fn test_long_match_before_short() {
        // "平行四边形" 应整体匹配，不应先被 "举行" 部分替换
        let output = correct_homophones("画一个平行四边形");
        assert!(output.contains("平行四边形"));
        assert!(!output.contains("平行四边矩形")); // 错误的部分替换
    }

    #[test]
    fn test_homophones_unchanged_when_correct() {
        // 正确文本不应被修改
        let input = "画一个红色矩形流程图";
        assert_eq!(correct_homophones(input), input);
    }
}
