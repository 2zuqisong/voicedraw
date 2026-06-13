# 指令系统重构 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 重构语音指令管道——扩展容错、增强 LLM prompt、新增操作预览确认机制，实现复杂图表的一句话说+预览+确认执行。

**Architecture:** 双路径管道：简单指令（≤3个工具调用）直接执行，复杂指令先规划→预览→确认→执行。九文件：四个 Rust (denoise, quick_match, system_prompt, scheduler)，一个 Rust 命令层 (commands/mod.rs)，三个前端 (store/types, store/index, OperationPreview.tsx)，一个配置 (Cargo.toml)。

**Tech Stack:** Rust (pinyin crate + thiserror) + TypeScript (React + Zustand) + DeepSeek LLM API

---

### Task 1: 扩展同音词纠正库 (15→100+ 对)

**Files:**
- Modify: `src-tauri/src/preprocessor/denoise.rs`

- [ ] **Step 1: 将 CORRECTIONS 数组从 ~15 对扩展到 100+ 对**

```rust
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
        ("指挥", "紫红"),  // 仅在颜色上下文
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
        ("ER图", "ER图"),
        ("翼啊图", "ER图"),
        ("UML图", "UML图"),
        ("优爱慕奥图", "UML图"),
        ("组织架构图", "组织架构图"),
        ("网络拓扑图", "网络拓扑图"),
        ("状态图", "状态图"),
        ("用力图", "用例图"),

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
        ("那就是", "就是说"),
        ("额那个", "那个"),
        ("然后呢", "然后"),
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
```

- [ ] **Step 2: 扩展测试以覆盖新词**

在 `src-tauri/src/preprocessor/denoise.rs` 的 `#[cfg(test)] mod tests` 中：

```rust
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
        assert!(output.contains(expected_in_output),
            "input='{}' should contain '{}', got '{}'", input, expected_in_output, output);
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
```

- [ ] **Step 3: 运行测试验证**

```bash
cargo test -p voice-draw denoise
```
Expected: 新增 5 个测试 + 原有 2 个测试全部 PASS

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/preprocessor/denoise.rs
git commit -m "feat: expand homophone correction from 15 to 100+ pairs

- Organize by category: graphics, colors, operations, diagram types,
  positions, connections, quantities, STT fragments
- Sort by length descending to prevent short matches from breaking
  longer phrases (e.g. 平行四边形 before 举行)
- Add 5 new tests covering graphics, colors, diagram types, ordering,
  and no-change-on-correct-input

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 2: 添加拼音模糊匹配 + 增强快捷指令容错

**Files:**
- Modify: `src-tauri/Cargo.toml`
- Modify: `src-tauri/src/preprocessor/quick_match.rs`

- [ ] **Step 1: 添加 pinyin crate 依赖**

在 `Cargo.toml` 的 `[dependencies]` 中添加：

```toml
pinyin = "0.10"
```

- [ ] **Step 2: 运行 cargo check 确认依赖可用**

```bash
cd src-tauri && cargo check
```
Expected: 编译成功（仅新增依赖，无功能代码变更）

- [ ] **Step 3: 实现拼音模糊匹配 + 增强 quick_match**

重写 `quick_match.rs`：

```rust
use pinyin::ToPinyin;
use serde_json::Value;

pub struct QuickAction {
    pub name: String,
    pub params: Value,
}

/// 快捷指令条目：关键词列表 + pinyin + 动作
struct Pattern {
    keywords: &'static [&'static str],
    pinyin: &'static str,    // 关键词拼音（用于 STT 中文输入的第二道防线）
    action: &'static str,
    params: Value,
}

/// 本地快捷指令匹配表
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
        .flat_map(|mut iter| iter.next().map(|p| p.plain().to_string()))
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
    for i in 0..=m { dp[i][0] = i; }
    for j in 0..=n { dp[0][j] = j; }

    for i in 1..=m {
        for j in 1..=n {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            dp[i][j] = (dp[i - 1][j] + 1)       // deletion
                .min(dp[i][j - 1] + 1)          // insertion
                .min(dp[i - 1][j - 1] + cost);  // substitution
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
```

- [ ] **Step 4: 运行所有 quick_match 测试**

```bash
cargo test -p voice-draw quick_match
```
Expected: 原有 4 个 + 新增 5 个 = 9 个测试全部 PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/Cargo.toml src-tauri/src/preprocessor/quick_match.rs
git commit -m "feat: add pinyin fuzzy matching and relax quick-match length tolerance

- Add pinyin crate dependency
- Two-pass matching: exact text match first, then pinyin Levenshtein fallback
- Relax keyword length tolerance from +5 to +10 characters
- Pinyin pass only for short inputs (<=10 chars) to prevent false positives
- Add 5 new tests: fuzzy length, pinyin matching, false positive guard, distance

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 3: 重写 System Prompt（三段式智能体手册 + 6 个 Few-shot 示例）

**Files:**
- Rewrite: `src-tauri/src/llm/system_prompt.rs`

- [ ] **Step 1: 编写完整的三段式 system prompt**

用以下内容**完整替换** `get_system_prompt()` 函数：

```rust
pub fn get_system_prompt() -> String {
    r#"你是一个专业的语音控制绘图助手。用户通过中文语音操控画布，你将自然语言精准转化为绘图操作。

## 行为准则
- 优先使用批量工具（add_nodes_batch + add_edges_batch），一次性完成所有创建操作
- 新流程图完成后自动调用 auto_layout 排列
- 不确定时简短反问确认，不猜测
- 回复简洁，每次不超过两句话
- 所有回复和标签使用中文
- 多个不相关的图必须放在不同网格区域，避免重叠

## 节点类型速查
- start/end — 开始/结束（圆角矩形）
- process — 普通步骤（矩形）
- decision — 判断分支（菱形）
- data — 数据/文件（平行四边形）
- subprocess — 子流程（矩形加粗边框）
- text — 纯文本标签

## 复杂指令处理
当你收到一个需要创建多个节点（3+）的复杂指令时：
1. 先在脑中规划：需要哪些节点、哪些连线、放在哪个网格位置
2. 如果用户未指定位置且画布已有内容，先调用 get_empty_anchor 查空位
3. 调用 add_nodes_batch 创建所有节点，带上 grid_x/grid_y 锚点
4. 调用 add_edges_batch 创建所有连线
5. 调用 auto_layout 自动排列
6. 简短回复结果

## 增量修改处理
当用户要求修改已有节点（改颜色、改文字、删除）时：
1. 从对话历史中找到目标节点的 ID 和当前属性
2. 如果不确定是哪个节点，反问确认（如"是要修改'验证'那个菱形吗？"）
3. 使用 update_node/delete_node 执行修改

## 网格坐标系统
- 1 格 = 20 像素，原点在画布左上角
- 用户说"在 (x, y) 处..."时填入 grid_x/grid_y（数字类型）
- 未指定位置时省略坐标参数，系统自动找空白位置
- 多图指令（如"先画A再画B"）：第一张图让系统自动定位，后续图调用 get_empty_anchor

---

## Few-shot 示例

### 示例 1：标准流程图
用户: "画用户登录流程：开始→输入账号密码→验证→成功/失败"
助手思考: 5个节点(Start/Process/Decision/End×2) + 4条连线，可批量完成。
工具调用:
  [add_nodes_batch] nodes=[
    {"type":"start","label":"开始"},
    {"type":"process","label":"输入账号密码"},
    {"type":"decision","label":"验证"},
    {"type":"end","label":"登录成功"},
    {"type":"end","label":"登录失败"}
  ]
  [add_edges_batch] edges=[
    {"from":"<node1>","to":"<node2>"},
    {"from":"<node2>","to":"<node3>"},
    {"from":"<node3>","to":"<node4>","label":"通过"},
    {"from":"<node3>","to":"<node5>","label":"不通过"}
  ]
  [auto_layout] direction="top_down"
回复: "已画出登录流程，5个节点4条连线。"

### 示例 2：含分支和退回的复杂流程
用户: "画审批流程：员工提交申请→部门经理审批→通过就总经理审批→不通过就退回修改→最后归档"
助手思考: 含分支和回退。5个节点 + 分支标签 + 退回连线。
工具调用:
  [add_nodes_batch] nodes=[
    {"type":"start","label":"开始"},
    {"type":"process","label":"员工提交申请"},
    {"type":"decision","label":"部门经理审批"},
    {"type":"process","label":"总经理审批"},
    {"type":"process","label":"退回修改"},
    {"type":"end","label":"归档"}
  ]
  [add_edges_batch] edges=[
    {"from":"<n1>","to":"<n2>"},
    {"from":"<n2>","to":"<n3>"},
    {"from":"<n3>","to":"<n4>","label":"通过"},
    {"from":"<n4>","to":"<n6>"},
    {"from":"<n3>","to":"<n5>","label":"不通过"},
    {"from":"<n5>","to":"<n2>"}
  ]
  [auto_layout]
回复: "已画出审批流程，含分支和退回逻辑。"

### 示例 3：架构图（多组件空间分配）
用户: "画微服务架构图：API网关、用户服务、订单服务、商品服务、消息队列、数据库"
助手思考: 架构图6个组件，按层次排列：网关在上、服务在中间、基础设施在下。先查空位。
工具调用:
  [get_empty_anchor]
    → grid(2,2)
  [add_nodes_batch] nodes=[
    {"type":"process","label":"API网关"},
    {"type":"process","label":"用户服务"},
    {"type":"process","label":"订单服务"},
    {"type":"process","label":"商品服务"},
    {"type":"data","label":"消息队列"},
    {"type":"data","label":"数据库"}
  ] grid_x=2 grid_y=2
  [add_edges_batch] edges=[网关→各服务, 各服务→队列, 各服务→数据库]
  [auto_layout]
回复: "已画出微服务架构图，6个组件按层次排列。"

### 示例 4：思维导图
用户: "画项目计划思维导图：中心是产品上线，分支有开发、测试、部署、运营"
助手思考: 思维导图 = 1中心 + 4分支，所有分支连线到中心。
工具调用:
  [add_nodes_batch] nodes=[
    {"type":"subprocess","label":"产品上线"},
    {"type":"process","label":"开发"},
    {"type":"process","label":"测试"},
    {"type":"process","label":"部署"},
    {"type":"process","label":"运营"}
  ]
  [add_edges_batch] edges=[中心→开发, 中心→测试, 中心→部署, 中心→运营]
  [auto_layout] direction="top_down"
回复: "已画出产品上线思维导图，1个中心4个分支。"

### 示例 5：增量修改
用户: "把验证那个菱形改成安全验证，颜色改橙色"
助手思考: 上一轮创建了"验证"决策节点（id=node_3），需要同时改label和fill。
工具调用:
  [update_node] node_id="node_3" label="安全验证" fill="#ff9800"
回复: "已将'验证'改为'安全验证'并设为橙色。"

### 示例 6：多图指令
用户: "先在左边画登录流程图，再在右边画数据库ER图"
助手思考: 两个独立图表必须分开放置。先画第一张，再查空位画第二张。
工具调用（第一轮）:
  [add_nodes_batch] nodes=[登录流程: 开始/输入/验证/成功/失败] grid_x=2 grid_y=2
  [add_edges_batch] edges=[流程连线]
  [auto_layout]
（第二轮）
  [get_empty_anchor] → grid(28,2)
  [add_nodes_batch] nodes=[ER图: 用户/订单/商品/评论] grid_x=28 grid_y=2
  [add_edges_batch] edges=[ER关系线]
  [auto_layout]
回复: "登录流程图在左侧，数据库ER图在右侧，两图不重叠。"

## 提醒
- 时刻记住这些示例的操作模式——批量创建→连线→布局
- 复杂的用户指令 = 多个简单示例的组合
- 当用户说"和刚才那个一样"时，复用上一轮的图表结构
"#.to_string()
}
```

- [ ] **Step 2: 编译检查**

```bash
cd src-tauri && cargo check
```
Expected: 编译成功

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/llm/system_prompt.rs
git commit -m "feat: rewrite system prompt as structured agent handbook with 6 few-shot examples

- Three-section structure: behavior rules, node types, grid system
- 6 few-shot examples covering: flowchart, branching, architecture,
  mindmap, incremental edit, multi-diagram
- Each example shows user_input → reasoning → tool_calls → response
- Add explicit instruction for complex command decomposition

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 4: 改造 LLM Scheduler（复杂度判断 + Plan 缓存）

**Files:**
- Modify: `src-tauri/src/llm/scheduler.rs`

- [ ] **Step 1: 添加复杂度判断 prompt 和 OperationPlan 结构体**

在 `scheduler.rs` 文件顶部（`use` 语句之后，`LLMScheduler` 之前）添加：

```rust
use crate::engine::grid::GridConfig;

/// 操作计划（用于预览确认）
#[derive(Debug, Clone, serde::Serialize)]
pub struct OperationPlan {
    pub id: String,
    pub diagram_type: String,
    pub summary: String,
    pub nodes: Vec<PlanNode>,
    pub edges: Vec<PlanEdge>,
    pub grid_position: Option<(f64, f64)>,
    pub layout_direction: String,
    pub estimated_tool_calls: u32,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlanNode {
    pub label: String,
    #[serde(rename = "type")]
    pub node_type: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PlanEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

/// 复杂度判断 prompt
const COMPLEXITY_PROMPT: &str = r#"分析以下用户指令的复杂度。只返回 JSON，不要其他内容。

判断标准:
- simple: 单节点添加、修改颜色/文字、删除、查询状态、撤销/重做、缩放/导出
- complex: 创建2+个新节点、新建流程图/架构图/思维导图/ER图、批量修改

返回格式:
{
  "complexity": "simple" 或 "complex",
  "reason": "简要原因（中文，一句话）",
  "estimated_tool_calls": 数字
}

指令: "#;

/// 执行阶段的 system prompt（简化版，因为计划已确认）
const EXECUTION_PROMPT: &str = r#"你是绘图执行助手。用户已确认操作计划，请按计划执行工具调用。
优先使用批量工具，所有标签用中文。执行完成后简短回复结果。"#;
```

- [ ] **Step 2: 修改 LLMScheduler 结构体，添加 plan_cache**

替换 `LLMScheduler` 的定义：

```rust
/// LLM 调度器：管理 DeepSeek API 多轮对话循环
pub struct LLMScheduler {
    client: DeepSeekClient,
    max_rounds: u8,
    plan_cache: Option<OperationPlan>,
}

impl LLMScheduler {
    pub fn new(api_key: String) -> Self {
        Self {
            client: DeepSeekClient::new(api_key, None),
            max_rounds: 5,
            plan_cache: None,
        }
    }
```

- [ ] **Step 3: 修改 process() 方法，先做复杂度判断**

在 `process()` 方法开头（`log::info!` 之后），插入复杂度判断逻辑。完整修改后的 `process()`:

```rust
    /// 处理用户指令，返回最终回复和更新的 Canvas 状态
    pub async fn process(
        &mut self,
        user_text: &str,
        history: &[(String, String)], // (role, content)
        engine: &AppEngine,
    ) -> Result<ProcessResult, String> {
        log::info!(
            "LLM Scheduler: 处理指令 '{}', 历史 {} 轮",
            user_text,
            history.len() / 2
        );

        // 1. 复杂度判断
        let complexity = self.judge_complexity(user_text).await?;
        log::info!(
            "复杂度判断: {} (预估 {} 个工具调用)",
            complexity.0,
            complexity.2
        );

        if complexity.0 == "complex" && complexity.2 > 3 {
            // 复杂指令：生成计划 → 等待用户确认
            let nodes = self.generate_plan_nodes(user_text).await?;
            let plan = OperationPlan {
                id: uuid::Uuid::new_v4().to_string(),
                diagram_type: self.infer_diagram_type(user_text),
                summary: complexity.1.clone(),
                nodes,
                edges: vec![],
                grid_position: {
                    let canvas = engine.canvas.lock().unwrap();
                    canvas.as_ref().map(|c| {
                        let grid_cfg = GridConfig::default();
                        let (gx, gy) = grid_cfg.find_empty_anchor(&c.nodes);
                        (gx, gy)
                    })
                },
                layout_direction: "top_down".into(),
                estimated_tool_calls: complexity.2,
            };
            let plan_json = serde_json::to_string(&plan).unwrap_or_default();
            log::info!("生成操作计划: id={}", plan.id);
            self.plan_cache = Some(plan);
            return Ok(ProcessResult::PendingPlan {
                plan_json,
                message: format!("📋 即将创建{}", complexity.1),
            });
        }

        // 简单指令：沿用现有多轮执行逻辑
        let result = self.execute_full(user_text, history, engine).await?;
        Ok(ProcessResult::Executed(result))
    }
```

注意：原有的 `process()` 方法签名从 `&self` 改为 `&mut self`。

- [ ] **Step 4: 添加辅助方法**

在 `impl LLMScheduler` 块中添加以下方法：

```rust
    /// 复杂度判断：轻量 LLM 调用
    async fn judge_complexity(&self, user_text: &str) -> Result<(String, String, u32), String> {
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content("你是指令复杂度分析器。只返回JSON。")
                .build()
                .map_err(|e| format!("{}", e))?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(format!("{}{}", COMPLEXITY_PROMPT, user_text))
                .build()
                .map_err(|e| format!("{}", e))?
                .into(),
        ];

        let response = self.client.chat(messages, vec![], false).await?;
        let content = response.content.unwrap_or_default();

        // 解析 JSON 响应
        let v: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| format!("复杂度判断 JSON 解析失败: {} | raw={}", e, content))?;

        Ok((
            v["complexity"].as_str().unwrap_or("simple").to_string(),
            v["reason"].as_str().unwrap_or("未知操作").to_string(),
            v["estimated_tool_calls"].as_u64().unwrap_or(1) as u32,
        ))
    }

    /// 从用户指令推断图表类型
    fn infer_diagram_type(&self, text: &str) -> String {
        if text.contains("流程") { return "流程图".into(); }
        if text.contains("架构") || text.contains("系统") { return "架构图".into(); }
        if text.contains("思维导图") || text.contains("脑图") { return "思维导图".into(); }
        if text.contains("ER") || text.contains("实体") { return "ER图".into(); }
        if text.contains("时序") { return "时序图".into(); }
        if text.contains("UML") { return "UML图".into(); }
        "图表".into()
    }

    /// 简单生成节点摘要（从用户文本中提取关键词作为节点标签）
    async fn generate_plan_nodes(&self, user_text: &str) -> Result<Vec<PlanNode>, String> {
        let messages = vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content("你是图表节点提取器。从用户描述中提取节点列表。只返回 JSON 数组。每个节点有 label（中文标签）和 type（start/end/process/decision/data/subprocess/text）。")
                .build()
                .map_err(|e| format!("{}", e))?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content(format!("提取此指令的所有节点: {}", user_text))
                .build()
                .map_err(|e| format!("{}", e))?
                .into(),
        ];

        let response = self.client.chat(messages, vec![], false).await?;
        let content = response.content.unwrap_or_default();
        let nodes: Vec<PlanNode> = serde_json::from_str(&content)
            .map_err(|e| format!("节点提取 JSON 解析失败: {} | raw={}", e, content))?;
        Ok(nodes)
    }

    /// 确认计划：执行完整 LLM 工具调用循环
    pub async fn confirm_plan(
        &mut self,
        user_text: &str,
        history: &[(String, String)],
        engine: &AppEngine,
    ) -> Result<SchedulerResult, String> {
        self.plan_cache = None; // 清除缓存
        self.execute_full(user_text, history, engine).await
    }

    /// 取消计划
    pub fn cancel_plan(&mut self) {
        self.plan_cache = None;
        log::info!("操作计划已取消");
    }

    /// 修改计划：等同于取消 + 重新处理
    pub fn modify_plan(&mut self) {
        self.plan_cache = None;
    }

    /// 完整执行（现有逻辑提取为独立方法）
    async fn execute_full(
        &self,
        user_text: &str,
        history: &[(String, String)],
        engine: &AppEngine,
    ) -> Result<SchedulerResult, String> {
        // [此处粘贴原有的 process() 中构建 messages → 多轮 tool_calls 循环的全部逻辑]
        // 保持和原来完全一样，只是从 process() 中提取出来
        // ...
        // 返回 SchedulerResult { message, canvas_state }
        todo!() // 占位 — 下一步实现
    }
```

- [ ] **Step 5: 提取 execute_full() — 将原有逻辑移入**

将原来 `process()` 中从"构建 messages"到"返回 SchedulerResult"的部分完整移入 `execute_full()`。替换 `todo!()` 占位符：

```rust
    async fn execute_full(
        &self,
        user_text: &str,
        history: &[(String, String)],
        engine: &AppEngine,
    ) -> Result<SchedulerResult, String> {
        let mut messages: Vec<ChatCompletionRequestMessage> = Vec::new();

        // 1. System prompt
        messages.push(
            ChatCompletionRequestSystemMessageArgs::default()
                .content(get_system_prompt())
                .build()
                .map_err(|e| format!("构建 system message 失败: {}", e))?
                .into(),
        );

        // 2. 历史对话（最近 5 轮）
        for (role, content) in history.iter().rev().take(5).rev() {
            match role.as_str() {
                "user" => {
                    messages.push(
                        ChatCompletionRequestUserMessageArgs::default()
                            .content(content.clone())
                            .build()
                            .map_err(|e| format!("构建 user message 失败: {}", e))?
                            .into(),
                    );
                }
                "assistant" => {
                    messages.push(
                        ChatCompletionRequestAssistantMessageArgs::default()
                            .content(content.clone())
                            .build()
                            .map_err(|e| format!("构建 assistant message 失败: {}", e))?
                            .into(),
                    );
                }
                _ => {}
            }
        }

        // 3. 对话历史摘要
        let conv_summary = if history.is_empty() {
            "（无历史对话）".into()
        } else {
            history
                .iter()
                .enumerate()
                .filter_map(|(_i, (role, content))| {
                    if role == "user" {
                        Some(format!("用户: {}", content))
                    } else {
                        Some(format!("助手: {}", content))
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        };

        // 4. 当前 canvas 状态摘要
        let canvas_summary = {
            let canvas = engine.canvas.lock().unwrap();
            canvas.as_ref().map(|c| {
                format!(
                    "当前画布: {}, 节点数: {}, 连线数: {}, 主题: {:?}",
                    c.title,
                    c.nodes.len(),
                    c.edges.len(),
                    c.theme
                )
            }).unwrap_or_default()
        };

        let user_msg = format!(
            "对话历史:\n{}\n\n用户指令: {}\n{}",
            conv_summary,
            user_text,
            if canvas_summary.is_empty() {
                "画布为空".into()
            } else {
                canvas_summary
            }
        );

        messages.push(
            ChatCompletionRequestUserMessageArgs::default()
                .content(user_msg)
                .build()
                .map_err(|e| format!("构建 user message 失败: {}", e))?
                .into(),
        );

        // 5. 工具定义
        let tools = get_tool_definitions();

        let mut final_content = String::new();

        // 6. 多轮循环（最多 max_rounds 轮）
        for round in 0..self.max_rounds {
            log::info!("LLM 第 {}/{} 轮...", round + 1, self.max_rounds);

            let response = self
                .client
                .chat(messages.clone(), tools.clone(), false)
                .await?;

            let has_tool_calls = response
                .tool_calls
                .as_ref()
                .map(|tc| !tc.is_empty())
                .unwrap_or(false);

            if !has_tool_calls {
                final_content = response.content.unwrap_or_else(|| "操作完成".into());
                log::info!("LLM 返回纯文本回复，结束循环: {}", final_content);
                break;
            }

            let tool_calls = response.tool_calls.as_ref().unwrap();
            log::info!(
                "第 {} 轮 LLM 请求 {} 个工具调用: {:?}",
                round + 1,
                tool_calls.len(),
                tool_calls.iter().map(|tc| tc.name.as_str()).collect::<Vec<_>>()
            );

            let openai_tool_calls: Vec<ChatCompletionMessageToolCall> = tool_calls
                .iter()
                .map(|tc| ChatCompletionMessageToolCall {
                    id: tc.id.clone(),
                    r#type: async_openai::types::ChatCompletionToolType::Function,
                    function: FunctionCall {
                        name: tc.name.clone(),
                        arguments: tc.arguments.clone(),
                    },
                })
                .collect();

            messages.push(
                ChatCompletionRequestAssistantMessageArgs::default()
                    .content(response.content.clone().unwrap_or_default())
                    .tool_calls(openai_tool_calls)
                    .build()
                    .map_err(|e| format!("构建 assistant tool_calls message 失败: {}", e))?
                    .into(),
            );

            for tc in tool_calls {
                let tool_result = execute_tool_call(engine, &tc.name, &tc.arguments)
                    .unwrap_or_else(|e| format!("错误: {}", e));

                messages.push(
                    ChatCompletionRequestToolMessageArgs::default()
                        .tool_call_id(tc.id.clone())
                        .content(tool_result)
                        .build()
                        .map_err(|e| format!("构建 tool message 失败: {}", e))?
                        .into(),
                );
            }

            if round == self.max_rounds - 1 {
                final_content = "已完成操作（达到最大轮次限制）".into();
            }
        }

        let canvas_state = engine.canvas.lock().unwrap().clone();

        Ok(SchedulerResult {
            message: final_content,
            canvas_state,
        })
    }
```

- [ ] **Step 6: 更新 ProcessResult 和 SchedulerResult 的导出**

在文件末尾（`execute_tool_call` 函数之前）更新类型定义：

```rust
/// process() 的返回类型
pub enum ProcessResult {
    Executed(SchedulerResult),
    PendingPlan {
        plan_json: String,
        message: String,
    },
}

pub struct SchedulerResult {
    pub message: String,
    pub canvas_state: Option<crate::engine::canvas_state::CanvasState>,
}
```

要移除原有的 `SchedulerResult` 定义（如果已存在）并替换为这个版本。确保 `ProcessResult` 在文件顶部可见（被 `commands/mod.rs` 引用）。

- [ ] **Step 7: 添加 missing imports**

在文件顶部的 use 块中确认有：
```rust
use uuid::Uuid;
```
如果没有则添加。检查 `crate::engine::grid::GridConfig` 是否正确导入。

- [ ] **Step 8: 编译检查**

```bash
cd src-tauri && cargo check 2>&1
```
Expected: 编译成功（或仅有 warning）

修复任何编译错误后重复此步骤。

- [ ] **Step 9: 运行现有测试**

```bash
cargo test -p voice-draw
```
Expected: 所有已有测试仍然 PASS

- [ ] **Step 10: Commit**

```bash
git add src-tauri/src/llm/scheduler.rs
git commit -m "feat: add complexity judgment and plan cache to LLM scheduler

- Add OperationPlan/PlanNode/PlanEdge types for preview confirmation
- judge_complexity(): lightweight LLM call to classify simple vs complex
- Complex commands (>3 estimated tool calls) generate plan, cache it,
  and return PendingPlan to frontend for user confirmation
- Simple commands execute immediately via extracted execute_full()
- Add confirm_plan/cancel_plan/modify_plan methods for plan lifecycle

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 5: 新增 Tauri 命令（confirm_plan / cancel_plan / modify_plan）

**Files:**
- Modify: `src-tauri/src/commands/mod.rs`

- [ ] **Step 1: 修改 process_command 以处理 ProcessResult**

在 `process_command` 中，将 `scheduler.process()` 的返回类型处理从 `SchedulerResult` 改为 `ProcessResult`：

```rust
// 将 scheduler 声明改为 mutable
let mut scheduler = llm::scheduler::LLMScheduler::new(api_key);

match scheduler.process(&enriched_text, &history, &ENGINE).await {
    Ok(llm::scheduler::ProcessResult::Executed(result)) => {
        log::info!("LLM 处理成功: {}", result.message);

        if let Some(ref state) = result.canvas_state {
            ENGINE.snapshots.lock().unwrap().save(state.clone());
            let _ = app.emit("canvas-updated", state);
        }

        {
            let mut ctx = ENGINE.context.lock().unwrap();
            ctx.add_turn(
                cleaned_text.clone(),
                result.message.clone(),
                vec![],
            );
        }

        Ok(serde_json::json!({
            "success": true,
            "message": result.message,
            "canvas_state": result.canvas_state,
            "pending_plan": null
        }))
    }
    Ok(llm::scheduler::ProcessResult::PendingPlan { plan_json, message }) => {
        log::info!("LLM 返回待确认计划");
        Ok(serde_json::json!({
            "success": true,
            "message": message,
            "canvas_state": null,
            "pending_plan": serde_json::from_str::<serde_json::Value>(&plan_json).unwrap_or(serde_json::Value::Null)
        }))
    }
    Err(e) => {
        // 保持原有错误处理不变
        log::error!("LLM 处理失败: {}", e);
        Ok(serde_json::json!({
            "success": false,
            "message": format!("LLM 处理失败: {}", e),
            "canvas_state": null,
            "pending_plan": null
        }))
    }
}
```

- [ ] **Step 2: 将 scheduler 提升为全局可访问**

为了在 `confirm_plan` / `cancel_plan` 中访问同一个 scheduler 实例，将 scheduler 存入全局：

在文件顶部（`ENGINE` 旁边）添加：

```rust
use std::sync::Mutex;

/// 全局 LLM 调度器（用于跨请求的 plan 缓存）
static LLM_SCHEDULER: std::sync::LazyLock<Mutex<Option<llm::scheduler::LLMScheduler>>> =
    std::sync::LazyLock::new(|| Mutex::new(None));
```

修改 `process_command` 中创建 scheduler 的方式：

```rust
let api_key = std::env::var("DEEPSEEK_API_KEY")
    .unwrap_or_else(|_| "sk-placeholder".into());

let mut scheduler_guard = LLM_SCHEDULER.lock().unwrap();
*scheduler_guard = Some(llm::scheduler::LLMScheduler::new(api_key));
let scheduler = scheduler_guard.as_mut().unwrap();

match scheduler.process(&enriched_text, &history, &ENGINE).await {
    // ... (同上)
}
```

- [ ] **Step 3: 新增三个 Tauri 命令**

在 `quick_action` 命令之后添加：

```rust
#[tauri::command]
pub async fn confirm_plan(app: tauri::AppHandle) -> Result<serde_json::Value, String> {
    log::info!("confirm_plan: 用户确认执行");
    let mut guard = LLM_SCHEDULER.lock().unwrap();
    let scheduler = guard.as_mut().ok_or("调度器未初始化")?;

    let history = {
        let ctx = ENGINE.context.lock().unwrap();
        ctx.to_history()
    };

    // 从上下文中获取原始用户文本
    let user_text = {
        let ctx = ENGINE.context.lock().unwrap();
        ctx.last_user_text().unwrap_or_default()
    };

    match scheduler.confirm_plan(&user_text, &history, &ENGINE).await {
        Ok(result) => {
            log::info!("计划执行成功: {}", result.message);
            if let Some(ref state) = result.canvas_state {
                ENGINE.snapshots.lock().unwrap().save(state.clone());
                let _ = app.emit("canvas-updated", state);
            }
            {
                let mut ctx = ENGINE.context.lock().unwrap();
                ctx.add_turn(user_text, result.message.clone(), vec![]);
            }
            Ok(serde_json::json!({
                "success": true,
                "message": result.message,
                "canvas_state": result.canvas_state
            }))
        }
        Err(e) => {
            log::error!("计划执行失败: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "message": format!("执行失败: {}", e),
                "canvas_state": null
            }))
        }
    }
}

#[tauri::command]
pub async fn cancel_plan() -> Result<serde_json::Value, String> {
    log::info!("cancel_plan: 用户取消计划");
    let mut guard = LLM_SCHEDULER.lock().unwrap();
    if let Some(ref mut scheduler) = *guard {
        scheduler.cancel_plan();
    }
    Ok(serde_json::json!({"success": true, "message": "已取消"}))
}

#[tauri::command]
pub async fn modify_plan(new_text: String) -> Result<serde_json::Value, String> {
    log::info!("modify_plan: 用户修改指令 '{}'", new_text);
    let mut guard = LLM_SCHEDULER.lock().unwrap();
    let scheduler = guard.as_mut().ok_or("调度器未初始化")?;
    scheduler.modify_plan();

    let history = {
        let ctx = ENGINE.context.lock().unwrap();
        ctx.to_history()
    };

    match scheduler.process(&new_text, &history, &ENGINE).await {
        Ok(llm::scheduler::ProcessResult::Executed(result)) => {
            Ok(serde_json::json!({
                "success": true,
                "message": result.message,
                "canvas_state": result.canvas_state,
                "pending_plan": null
            }))
        }
        Ok(llm::scheduler::ProcessResult::PendingPlan { plan_json, message }) => {
            Ok(serde_json::json!({
                "success": true,
                "message": message,
                "canvas_state": null,
                "pending_plan": serde_json::from_str::<serde_json::Value>(&plan_json).unwrap_or(serde_json::Value::Null)
            }))
        }
        Err(e) => {
            Ok(serde_json::json!({
                "success": false,
                "message": format!("处理失败: {}", e),
                "canvas_state": null,
                "pending_plan": null
            }))
        }
    }
}
```

- [ ] **Step 4: 在 lib.rs 中注册新命令**

修改 `src-tauri/src/lib.rs`，在 `invoke_handler` 中注册新命令：

找到类似以下的代码：
```rust
.invoke_handler(tauri::generate_handler![
    commands::process_command,
    commands::quick_action,
])
```

改为：
```rust
.invoke_handler(tauri::generate_handler![
    commands::process_command,
    commands::quick_action,
    commands::confirm_plan,
    commands::cancel_plan,
    commands::modify_plan,
])
```

- [ ] **Step 5: 确保 ConversationContext 有 last_user_text()**

检查 `src-tauri/src/preprocessor/context_enrich.rs`，如果没有 `last_user_text()` 方法则需要添加。先检查：

```bash
grep -n "last_user_text" src-tauri/src/preprocessor/context_enrich.rs
```

如果不存在，在 `impl ConversationContext` 中添加：

```rust
/// 获取最近一次用户输入的文本
pub fn last_user_text(&self) -> Option<String> {
    self.turns.last().map(|t| t.user_text.clone())
}
```

- [ ] **Step 6: 编译检查**

```bash
cd src-tauri && cargo check 2>&1
```
Expected: 编译成功

修复任何编译错误后重复此步骤。

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/commands/mod.rs src-tauri/src/lib.rs src-tauri/src/preprocessor/context_enrich.rs
git commit -m "feat: add confirm_plan/cancel_plan/modify_plan Tauri commands

- process_command now handles ProcessResult::PendingPlan for preview flow
- Global LLM_SCHEDULER static for cross-request plan cache access
- confirm_plan: resumes execution with cached plan
- cancel_plan: clears cached plan
- modify_plan: re-processes with new instruction text
- Add last_user_text() to ConversationContext

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 6: 前端 Store + Types 扩展

**Files:**
- Modify: `src/store/types.ts`
- Modify: `src/store/index.ts`

- [ ] **Step 1: 添加 OperationPlan 类型**

在 `src/store/types.ts` 末尾添加：

```typescript
/** 操作计划（预览确认用） */
export interface OperationPlan {
  id: string;
  diagram_type: string;
  summary: string;
  nodes: PlanNode[];
  edges: PlanEdge[];
  grid_position: [number, number] | null;
  layout_direction: "top_down" | "left_right";
  estimated_tool_calls: number;
}

export interface PlanNode {
  label: string;
  type: string;
}

export interface PlanEdge {
  from: string;
  to: string;
  label: string | null;
}
```

- [ ] **Step 2: 更新 OperationResult 接口**

在同一个文件中更新 `OperationResult`，加入 `pending_plan` 字段：

```typescript
/** LLM 工具调用的操作结果 */
export interface OperationResult {
  success: boolean;
  message: string;
  canvas_state: CanvasState | null;
  pending_plan: OperationPlan | null;
}
```

- [ ] **Step 3: 扩展 Zustand Store**

在 `src/store/index.ts` 中：

添加 import：
```typescript
import type { CanvasState, AppStatus, ConversationMessage, OperationResult, OperationPlan } from "./types";
```

在 `AppState` 接口中添加：
```typescript
  // 操作计划
  pendingPlan: OperationPlan | null;
  confirmPlan: () => Promise<void>;
  cancelPlan: () => Promise<void>;
  modifyPlan: (newText: string) => Promise<void>;
```

在 `create` 的初始状态中添加：
```typescript
  pendingPlan: null,
```

在 `submitCommand` 中，处理 `pending_plan` 返回：

修改 `submitCommand` 的 `.then()` 部分：
```typescript
  submitCommand: async (text) => {
    set({ status: "thinking", lastOperation: text });
    try {
      const result = await invoke<OperationResult>("process_command", { text });
      if (result.pending_plan) {
        // 复杂指令：显示预览面板
        set({
          pendingPlan: result.pending_plan,
          status: "idle",
          lastOperation: result.message,
        });
      } else if (result.success && result.canvas_state) {
        set({
          canvasState: result.canvas_state,
          status: "idle",
          lastOperation: result.message,
          conversation: [
            ...get().conversation,
            { role: "user" as const, content: text },
            { role: "assistant" as const, content: result.message },
          ].slice(-10),
        });
      } else {
        set({ status: "error", lastOperation: result.message || "操作失败" });
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      set({ status: "error", lastOperation: `错误: ${errorMsg}` });
      setTimeout(() => set({ status: "idle" }), 3000);
    }
  },
```

在 `quickAction` 之后添加三个新的 action：
```typescript
  confirmPlan: async () => {
    const plan = get().pendingPlan;
    if (!plan) return;
    set({ status: "executing", lastOperation: `执行: ${plan.summary}` });
    try {
      const result = await invoke<OperationResult>("confirm_plan");
      if (result.success && result.canvas_state) {
        set({
          canvasState: result.canvas_state,
          pendingPlan: null,
          status: "idle",
          lastOperation: result.message,
          conversation: [
            ...get().conversation,
            { role: "user" as const, content: `确认执行: ${plan.summary}` },
            { role: "assistant" as const, content: result.message },
          ].slice(-10),
        });
      } else {
        set({ status: "error", pendingPlan: null, lastOperation: result.message || "执行失败" });
        setTimeout(() => set({ status: "idle" }), 3000);
      }
    } catch (err) {
      set({ status: "error", pendingPlan: null, lastOperation: String(err) });
      setTimeout(() => set({ status: "idle" }), 3000);
    }
  },

  cancelPlan: async () => {
    try {
      await invoke("cancel_plan");
    } catch { /* 忽略 */ }
    set({ pendingPlan: null, status: "idle", lastOperation: "已取消" });
  },

  modifyPlan: async (newText) => {
    const plan = get().pendingPlan;
    if (!plan) return;
    set({ status: "thinking", lastOperation: newText });
    try {
      const result = await invoke<OperationResult>("modify_plan", { newText });
      if (result.pending_plan) {
        set({ pendingPlan: result.pending_plan, status: "idle", lastOperation: result.message });
      } else if (result.success && result.canvas_state) {
        set({
          canvasState: result.canvas_state,
          pendingPlan: null,
          status: "idle",
          lastOperation: result.message,
        });
      } else {
        set({ status: "error", pendingPlan: null, lastOperation: result.message || "修改失败" });
        setTimeout(() => set({ status: "idle" }), 3000);
      }
    } catch (err) {
      set({ status: "error", pendingPlan: null, lastOperation: String(err) });
      setTimeout(() => set({ status: "idle" }), 3000);
    }
  },
```

- [ ] **Step 4: TypeScript 编译检查**

```bash
npx tsc --noEmit --pretty
```
Expected: No errors

- [ ] **Step 5: Commit**

```bash
git add src/store/types.ts src/store/index.ts
git commit -m "feat: add OperationPlan types and pendingPlan store actions

- Add OperationPlan, PlanNode, PlanEdge TypeScript interfaces
- Add pending_plan field to OperationResult
- submitCommand now handles pending_plan for preview flow
- Add confirmPlan/cancelPlan/modifyPlan store actions
- confirmPlan sends confirmed plan to backend, cancelPlan clears,
  modifyPlan re-submits with modified instruction text

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 7: 新建 OperationPreview 组件 + 接入 App

**Files:**
- Create: `src/components/canvas/OperationPreview.tsx`
- Modify: `src/App.tsx`

- [ ] **Step 1: 创建 OperationPreview 组件**

写入 `src/components/canvas/OperationPreview.tsx`：

```tsx
import { useAppStore } from "../../store";

export default function OperationPreview() {
  const pendingPlan = useAppStore((s) => s.pendingPlan);
  const confirmPlan = useAppStore((s) => s.confirmPlan);
  const cancelPlan = useAppStore((s) => s.cancelPlan);
  const modifyPlan = useAppStore((s) => s.modifyPlan);
  const [editText, setEditText] = useState("");
  const [isEditing, setIsEditing] = useState(false);

  if (!pendingPlan) return null;

  const gridPos = pendingPlan.grid_position
    ? `(${pendingPlan.grid_position[0]}, ${pendingPlan.grid_position[1]})`
    : "自动";

  return (
    <div style={{
      position: "absolute",
      top: 60,
      left: "50%",
      transform: "translateX(-50%)",
      zIndex: 200,
      background: "rgba(255,255,255,0.96)",
      borderRadius: 16,
      boxShadow: "0 4px 24px rgba(0,0,0,0.12), 0 1px 8px rgba(0,0,0,0.08)",
      padding: "16px 20px",
      minWidth: 380,
      maxWidth: 480,
      backdropFilter: "blur(8px)",
    }}>
      {/* 标题 */}
      <div style={{
        display: "flex",
        alignItems: "center",
        gap: 8,
        marginBottom: 12,
      }}>
        <span style={{ fontSize: 18 }}>📋</span>
        <span style={{
          fontSize: 14,
          fontWeight: 600,
          color: "#202124",
        }}>
          即将执行的操作
        </span>
      </div>

      {/* 操作摘要 */}
      <div style={{
        fontSize: 13,
        color: "#5f6368",
        lineHeight: 1.6,
        marginBottom: 8,
      }}>
        <div>
          <span style={{ color: "#9aa0a6" }}>类型：</span>
          {pendingPlan.diagram_type}
        </div>
        <div>
          <span style={{ color: "#9aa0a6" }}>节点：</span>
          {pendingPlan.nodes.map((n) => n.label).join("、")}
          <span style={{ color: "#9aa0a6" }}>
            （共 {pendingPlan.nodes.length} 个）
          </span>
        </div>
        {pendingPlan.edges.length > 0 && (
          <div>
            <span style={{ color: "#9aa0a6" }}>连线：</span>
            {pendingPlan.edges
              .slice(0, 3)
              .map(
                (e) =>
                  `${e.from}→${e.to}${e.label ? `(${e.label})` : ""}`
              )
              .join(", ")}
            {pendingPlan.edges.length > 3 &&
              ` ... 等${pendingPlan.edges.length}条`}
          </div>
        )}
        <div>
          <span style={{ color: "#9aa0a6" }}>位置：</span>
          网格 {gridPos}
        </div>
        <div>
          <span style={{ color: "#9aa0a6" }}>布局：</span>
          {pendingPlan.layout_direction === "top_down" ? "上→下" : "左→右"}
        </div>
      </div>

      {/* 操作按钮 */}
      {isEditing ? (
        <div style={{ display: "flex", gap: 8 }}>
          <input
            type="text"
            value={editText}
            onChange={(e) => setEditText(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                modifyPlan(editText);
                setEditText("");
                setIsEditing(false);
              }
              if (e.key === "Escape") {
                setIsEditing(false);
                setEditText("");
              }
            }}
            placeholder="修改指令后按回车..."
            autoFocus
            style={{
              flex: 1,
              height: 36,
              borderRadius: 20,
              border: "1px solid #667eea",
              padding: "0 14px",
              fontSize: 13,
              outline: "none",
            }}
          />
          <button
            onClick={() => {
              modifyPlan(editText);
              setEditText("");
              setIsEditing(false);
            }}
            style={{
              padding: "0 16px",
              height: 36,
              borderRadius: 20,
              border: "none",
              background: "#667eea",
              color: "#fff",
              fontSize: 13,
              cursor: "pointer",
            }}
          >
            修改
          </button>
        </div>
      ) : (
        <div style={{
          display: "flex",
          gap: 10,
          justifyContent: "center",
        }}>
          <button
            onClick={confirmPlan}
            style={{
              flex: 1,
              height: 36,
              borderRadius: 20,
              border: "none",
              background: "#34a853",
              color: "#fff",
              fontSize: 14,
              fontWeight: 500,
              cursor: "pointer",
            }}
          >
            ✓ 确认执行
          </button>
          <button
            onClick={cancelPlan}
            style={{
              width: 72,
              height: 36,
              borderRadius: 20,
              border: "1px solid #e8eaed",
              background: "#fff",
              color: "#ea4335",
              fontSize: 13,
              cursor: "pointer",
            }}
          >
            ✕ 取消
          </button>
          <button
            onClick={() => {
              setEditText("");
              setIsEditing(true);
            }}
            style={{
              width: 72,
              height: 36,
              borderRadius: 20,
              border: "1px solid #e8eaed",
              background: "#fff",
              color: "#5f6368",
              fontSize: 13,
              cursor: "pointer",
            }}
          >
            ✎ 修改
          </button>
        </div>
      )}
    </div>
  );
}
```

注意：需要添加 `useState` import：
```tsx
import { useState } from "react";
```

- [ ] **Step 2: 在 App.tsx 中接入 OperationPreview**

读取 `src/App.tsx`，在 CanvasView 的上方或同级添加 `<OperationPreview />`：

```tsx
import OperationPreview from "./components/canvas/OperationPreview";

// 在 JSX 中 CanvasView 上方添加：
<OperationPreview />
```

具体位置在 `CanvasView` 的容器 div 内部或 App 的根 div 中，使其浮在画布上方。

- [ ] **Step 3: TypeScript 编译检查**

```bash
npx tsc --noEmit --pretty
```
Expected: No errors

- [ ] **Step 4: Commit**

```bash
git add src/components/canvas/OperationPreview.tsx src/App.tsx
git commit -m "feat: add OperationPreview panel for plan confirmation

- Floating panel shows diagram type, nodes, edges, grid position, layout
- Three buttons: confirm (execute), cancel (discard), modify (edit text)
- Modify mode expands inline text input with keyboard support
- Auto-hides when no pending plan

Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

### Task 8: 端到端验证

- [ ] **Step 1: 运行全部 Rust 测试**

```bash
cd src-tauri && cargo test
```
Expected: All tests PASS

- [ ] **Step 2: 运行 TypeScript 编译检查**

```bash
npx tsc --noEmit --pretty
```
Expected: No errors

- [ ] **Step 3: 启动应用做手动冒烟测试**

```bash
DEEPSEEK_API_KEY=sk-xxx RUST_LOG=info bun run tauri dev
```

验证清单：
- [ ] 简单指令（"撤销""放大"）→ 直接执行，无预览
- [ ] 修改指令（"把节点改成蓝色"）→ 直接执行
- [ ] 复杂指令（"画用户登录流程图"）→ 弹预览面板
- [ ] 点击 ✓ 确认 → 画布正确渲染
- [ ] 点击 ✕ 取消 → 面板消失，画布不变
- [ ] 点击 ✎ 修改 → 输入框出现，输入新指令回车 → 重新规划
- [ ] 多图指令 → 两图不重叠
- [ ] 带坐标指令（"在(5,3)画矩形"）→ 预览中显示正确坐标

- [ ] **Step 4: 最终 Commit**

```bash
git add -A
git commit -m "chore: final verification - all tests pass, manual smoke test complete

Co-Authored-By: Claude <noreply@anthropic.com>"
```
