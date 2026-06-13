# 指令系统重构设计文档

> 重构语音控制绘图工具的指令理解管道，提升准确性、容错性和复杂图表绘制能力。

**问题陈述**：当前系统 LLM prompt 过于简短（8条规则），同音词纠错覆盖不足（~15对），缺乏预览确认机制。导致：复杂图生成不稳定、语音识别错误直接污染执行、用户对即将执行的操作无感知。

**目标**：改造为双路径 + 预览确认的智能指令管道，支持"一句话描述复杂图 → 预览 → 确认执行"的交互模式。

**技术栈**：Rust (Tauri 后端) + TypeScript/React (前端) + DeepSeek LLM API

---

## 1. 管道架构

```
语音输入
  → 去噪纠错（扩展同音词库 + 拼音模糊匹配）
    → 快捷匹配（增强容错）
      → 匹配成功 → 直接执行（简单操作如撤销/缩放）
      → 未匹配 → LLM 处理
        → 复杂度判断（单轮调用）
          → 简单指令（≤3 个工具调用）→ 直接执行 → 渲染
          → 复杂指令（>3 个工具调用）
            → 规划阶段：LLM 生成操作计划
            → 前端显示预览摘要（OperationPreview 面板）
            → 用户确认（✓ 执行 / ✕ 取消 / ✎ 修改指令）
            → 执行阶段：LLM 按计划调用工具 → 渲染
```

### 关键设计决策

- **为什么是双路径**：简单操作（改颜色、删节点）走预览会增加不必要的交互成本；复杂操作走预览防止不可逆的错误
- **为什么复杂度阈值是 ≤3 个工具调用**：经验值。单个 add_node + add_edge + update_node = 3 个调用。超过此值的操作通常涉及多个独立图表或大型流程图，值得确认
- **为什么规划用独立 prompt**：规划 prompt 要求 LLM 输出结构化计划文本而非直接调工具，与执行 prompt 关注点不同

---

## 2. System Prompt 重构

当前 8 条简短规则 → 改为 **结构化智能体手册**（三部分）

### 2.1 角色定位与行为准则

```
你是专业的语音控制绘图助手。用户通过中文语音操控画布，
你将自然语言精准转化为绘图操作。

行为准则：
- 优先批量工具（add_nodes_batch/add_edges_batch），减少调用轮次
- 不确定时反问确认，不猜测
- 回复简洁，每次不超过两句话
- 所有回复和标签使用中文
- 多个不相关的图必须放在不同网格区域，避免重叠
```

### 2.2 节点类型速查（保持不变）

```
- start/end → 开始/结束（圆角矩形）
- process → 普通步骤（矩形）
- decision → 判断分支（菱形）
- data → 数据/文件（平行四边形）
- subprocess → 子流程（矩形加粗边框）
- text → 纯文本标签
```

### 2.3 Few-shot 示例（核心新增，6 个）

每个示例严格遵循格式：`用户指令 → 思考过程 → 工具调用（JSON）→ 回复`

#### 示例 1：简单流程图

```
用户: "画用户登录流程：开始→输入账号密码→验证→成功/失败"

思考: 这是标准流程图。5个节点(开始/过程/判断/结束×2) + 4条连线。
     可批量创建后自动布局。用户未指定位置，省略grid参数让系统自动找空位。

工具调用序列:
  [add_nodes_batch] nodes=[
    {"type":"start","label":"开始"},
    {"type":"process","label":"输入账号密码"},
    {"type":"decision","label":"验证"},
    {"type":"end","label":"登录成功"},
    {"type":"end","label":"登录失败"}
  ]
  [add_edges_batch] edges=[
    {"from":"<node1_id>","to":"<node2_id>"},
    {"from":"<node2_id>","to":"<node3_id>"},
    {"from":"<node3_id>","to":"<node4_id>","label":"通过"},
    {"from":"<node3_id>","to":"<node5_id>","label":"不通过"}
  ]
  [auto_layout] direction="top_down"

回复: "已画出用户登录流程，共5个节点4条连线。"
```

#### 示例 2：含分支和循环的复杂流程

```
用户: "画审批流程：员工提交申请→部门经理审批→通过就总经理审批→不通过就退回修改→最后归档"

思考: 含分支+回退。5个节点(开始/过程/判断/过程/结束) + 1条回退连线。
     decision节点"部门经理审批"有两个出口。退回修改需要连线回第2个节点。

工具调用序列:
  [add_nodes_batch] nodes=[5个节点]
  [add_edges_batch] edges=[
    {"from":"n1","to":"n2"},
    {"from":"n2","to":"n3","label":"通过"},
    {"from":"n3","to":"n4"},
    {"from":"n4","to":"n5"},
    {"from":"n2","to":"n3_实际上这是退回","label":"不通过→退回修改"}
  ]
  // 实际实现中"退回修改"连回"提交申请"节点
  [auto_layout]

回复: "已画出审批流程，包含分支和退回逻辑。"
```

#### 示例 3：架构图（空间分配）

```
用户: "画一个微服务架构图：API网关、用户服务、订单服务、商品服务、消息队列、数据库"

思考: 这是架构图，6个组件各自独立，需要用网格坐标分区放置。
     先查空位，然后按层次排列：网关在上，服务在中间，基础设施在下方。

工具调用序列:
  [get_empty_anchor]
    → 返回 grid(2,2)
  [add_nodes_batch] nodes=[6个节点] grid_x=2 grid_y=2
  [add_edges_batch] edges=[网关→各服务, 服务→队列, 服务→数据库]
  [auto_layout] direction="top_down"

回复: "已画出微服务架构图，6个组件按层次排列。"
```

#### 示例 4：增量修改

```
用户: "把那个菱形的验证改成安全验证，颜色改成橙色"

思考: 从对话历史找节点——上一轮创建了id为node_3的decision节点"验证"。
     需要同时改label和fill。

工具调用序列:
  [update_node] node_id="node_3" label="安全验证" fill="#ff9800"

回复: "已将'验证'改为'安全验证'并设为橙色。"
```

#### 示例 5：思维导图

```
用户: "画项目计划思维导图：中心是产品上线，分支有开发、测试、部署、运营"

思考: 思维导图 = 中心节点 + 4个一级分支节点，全部连到中心。
     用subprocess类型表示中心主题，process表示分支。

工具调用序列:
  [add_nodes_batch] nodes=[中心+4分支]
  [add_edges_batch] edges=[中心→每个分支]
  [auto_layout] direction="top_down"

回复: "已画出产品上线思维导图，1个中心主题4个分支。"
```

#### 示例 6：多图指令

```
用户: "先在左上角画登录流程图，再在右边画一个数据库ER图"

思考: 两个完全独立的图，必须放在不同位置避免重叠。
     第一张图放网格(2,2)，第二张图用get_empty_anchor查询新空位。

工具调用序列（第一轮）:
  [add_nodes_batch] nodes=[登录流程节点] grid_x=2 grid_y=2
  [add_edges_batch] edges=[流程连线]
  [auto_layout]

  （第二轮）
  [get_empty_anchor]
    → 返回 grid(25,2)
  [add_nodes_batch] nodes=[ER图节点: 用户/订单/商品/评论] grid_x=25 grid_y=2
  [add_edges_batch] edges=[ER连线]
  [auto_layout]

回复: "登录流程图在左侧，数据库ER图在右侧，两图不重叠。"
```

---

## 3. 操作预览机制

### 3.1 触发条件

| 条件 | 行为 |
|------|------|
| 快捷匹配命中 | 直接执行，不预览 |
| LLM 判断 ≤3 个工具调用 | 直接执行，不预览 |
| LLM 判断 >3 个工具调用 | 生成计划 → 预览面板 |
| 用户说"预览"/"先看看" | 强制进入预览 |

### 3.2 前端 OperationPreview 组件

在画布上方弹出半透明面板，显示操作摘要：

```
┌─────────────────────────────────────────────┐
│  📋 即将执行的操作                            │
│                                              │
│  类型: 流程图                                 │
│  节点: 开始、输入验证、处理订单、结束 (共4个)    │
│  连线: 开始→输入验证, 输入验证→处理订单, ...    │
│  位置: 网格 (2, 2)                           │
│  布局: 上→下自动排列                          │
│                                              │
│  [✓ 确认执行]   [✕ 取消]   [✎ 修改指令]       │
└─────────────────────────────────────────────┘
```

### 3.3 三种确认结果

| 按钮 | 行为 |
|------|------|
| ✓ 确认执行 | LLM 继续按计划调用工具，画布渲染 |
| ✕ 取消 | 丢弃计划，状态回到 idle |
| ✎ 修改指令 | 展开编辑区，用户可补充/修正指令文本，重新规划 |

### 3.4 无超时

预览面板不设超时，一直等待用户操作。

### 3.5 计划数据结构

```typescript
interface OperationPlan {
  id: string;
  diagram_type: string;        // "流程图" | "架构图" | "思维导图" | ...
  summary: string;              // 一句话描述
  nodes: { label: string; type: string }[];
  edges: { from: string; to: string; label?: string }[];
  grid_position: { x: number; y: number } | null;
  layout_direction: "top_down" | "left_right";
  estimated_tool_calls: number;
}
```

---

## 4. 容错增强

### 4.1 同音词库扩展（15 → 100+ 对）

按类别组织，存储在 `denoise.rs` 中：

| 类别 | 数量 | 示例 |
|------|------|------|
| 图形词 | ~20 | 举行→矩形, 话圆→画圆, 灵星→菱形, 箭偷→箭头, 山角→三角, 拖延→椭圆, 六边→六边形 |
| 颜色词 | ~15 | 黄涩→黄色, 红设→红色, 旅色→绿色, 蓝设→蓝色, 层色→橙色, 指挥→紫红, 非得→飞得（误识别，忽略） |
| 操作词 | ~15 | 扯消→撤销, 中坐→重做, 抱存→保存, 打倒→导出, 放打→放大, 缩笑→缩小 |
| 图类词 | ~10 | 留成图→流程图, 司维导图→思维导图, 架构徒→架构图, 持续图→时序图, UML→UML |
| 位置/方向词 | ~10 | 昨边→左边, 又边→右边, 上放→上方, 下放→下方, 总间→中间 |
| 连接词 | ~10 | 亮解→连接, 连县→连线, 职向→指向, 关节→关系 |
| 数量词 | ~8 | 衣阁→一个, 良阁→两个, 三阁→三个, 几阁→几个 |
| 常见 STT 碎片 | ~15 | 的化→的话, 那就是→就是说, 额→嗯（去噪） |

### 4.2 拼音模糊匹配（新增函数）

对于快捷指令匹配失败的情况，做拼音二次匹配：

```
输入 → pinyin crate 转拼音（忽略声调）
     → 与快捷指令关键词拼音比较
     → 编辑距离 ≤ 2 且相似度最高 → 视为匹配
     → 否则 → 不匹配
```

例如：用户说"che xiao"但 STT 识别为"扯消"→ 拼音都是 `che xiao` → 匹配到"撤销"

### 4.3 指令确认兜底

LLM 在 system prompt 中被指示：如果理解指令时存在歧义，回复末尾加简短确认句。例如：

- "是要画一个4节点的流程图吗？"
- "要把所有节点改成蓝色还是只改刚才那个？"

---

## 5. LLM Scheduler 改造

### 5.1 当前流程

```
用户指令 → 构建 messages → LLM 多轮 tool_calls 循环 → 返回结果
```

### 5.2 改造后流程

```
用户指令
  → 复杂度判断（单轮 LLM 调用，轻量 prompt）
    → 返回: { complexity: "simple" | "complex", plan?: OperationPlan }
    → simple → 现有流程（多轮 tool_calls 直接执行）
    → complex → 缓存 plan → 返回给前端预览
      → 前端 confirm: 继续 LLM tool_calls 执行
      → 前端 cancel: 清除缓存
      → 前端 modify: 更新指令文本，重新判断
```

### 5.3 复杂度判断 Prompt

独立于执行 prompt 的轻量判断 prompt：

```
分析以下用户指令的复杂度，返回JSON:
{
  "complexity": "simple" 或 "complex",
  "reason": "简要原因",
  "estimated_tool_calls": 数字
}

判断标准:
- simple: 单个修改操作(改颜色/删节点/移动)、单个节点添加、查询画布状态
- complex: 创建2+个节点、新建流程图/架构图/思维导图、批量修改
```

### 5.4 新增 Tauri 命令

```rust
// commands/mod.rs 新增

#[tauri::command]
async fn confirm_plan(state: State<AppState>) -> Result<OperationResult, String> {
    // 从缓存获取 plan，继续 LLM 执行
}

#[tauri::command]
async fn cancel_plan(state: State<AppState>) -> Result<(), String> {
    // 清除缓存的 plan
}

#[tauri::command]
async fn modify_plan(state: State<AppState>, new_text: String) -> Result<OperationResult, String> {
    // 用新文本重新做复杂度判断+规划
}
```

---

## 6. 文件变更清单

| 文件 | 操作 | 改动内容 |
|------|------|----------|
| `src-tauri/src/preprocessor/denoise.rs` | 修改 | 同音词库 15→100+ 对；新增 `fuzzy_match_pinyin()` 函数 |
| `src-tauri/src/preprocessor/quick_match.rs` | 修改 | 加入拼音容错回退；放宽长度限制 |
| `src-tauri/src/llm/system_prompt.rs` | 重写 | 三段式智能体手册 + 6 个 few-shot 示例 |
| `src-tauri/src/llm/scheduler.rs` | 修改 | 新增复杂度判断；plan 缓存；confirm/cancel/modify 方法 |
| `src-tauri/src/llm/tool_defs.rs` | 不变 | 工具定义保持不变，复杂度判断通过独立 prompt 完成 |
| `src-tauri/src/commands/mod.rs` | 修改 | 新增 `confirm_plan`/`cancel_plan`/`modify_plan` 命令 |
| `src/components/canvas/OperationPreview.tsx` | **新建** | 操作预览面板 UI 组件 |
| `src/store/index.ts` | 修改 | 新增 `pendingPlan`、`confirmPlan`、`cancelPlan` |
| `src/store/types.ts` | 修改 | 新增 `OperationPlan` 接口 |

---

## 7. 风险与缓解

| 风险 | 缓解 |
|------|------|
| 复杂度判断误判（简单指令被判为复杂） | 即使误判，用户多点击一次确认即可，不会产生错误 |
| 复杂度判断漏判（复杂指令被判为简单） | 阈值设得较低（≤3个tool calls），流程图几乎必然 >3 |
| Few-shot 示例占用 token | 6 个示例约 1500 tokens，DeepSeek 上下文窗口足够 |
| 拼音模糊匹配误匹配 | 编辑距离阈值 ≤2 且只在快捷匹配失败时启用，误匹配率低 |
| 预览面板打断语音流 | 确认按钮大且醒目，一次点击完成，不增加语音负担 |
