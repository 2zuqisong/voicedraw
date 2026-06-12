# Voice-to-Draw: 纯语音智能绘图系统 — 设计文档

> 日期: 2026-06-12
> 状态: 设计阶段
> 技术栈: Tauri + React + Rust + DeepSeek V4

---

## 1. 产品定义

### 1.1 核心目标

一款**纯语音控制**的智能绘图工具。用户不使用鼠标或键盘，仅通过中文语音指令完成图表/流程图的创作。系统通过 LLM 理解用户意图，自动选择图表类型，拆解复杂指令，并以低延迟渲染到 Canvas。

### 1.2 关键约束

| 约束 | 要求 |
|------|------|
| 交互方式 | 纯语音，无鼠标/键盘（开发者调试除外） |
| 语言 | 中文 |
| 图表类型 | AI 自动判断（流程图/思维导图/架构图/UML/ER图等） |
| 指令复杂度 | D 级：复杂多步指令 + 迭代修改 + 对话式交互 |
| 延迟目标 | 简单指令 < 500ms，复杂指令 2-4s |
| LLM | DeepSeek V4（云端 API） |
| STT（MVP） | Web Speech API |
| STT（最终） | Web Speech API + Whisper.cpp（可选本地兜底） |

### 1.3 典型使用场景

- **创建图表**：「画一个用户登录流程图，包含手机号和验证码两种方式，失败3次锁定」
- **迭代修改**：「把手机号登录改成绿色」「在风控检查后面加一个发送通知的步骤」「重新排版成从左到右」
- **快捷操作**：「撤销」「放大」「导出 PNG」「清空画布」

---

## 2. 系统架构

### 2.1 三层架构总览

```
┌────────────────────────────────────────────────────────┐
│                   🖥️  React 前端层                      │
│                                                        │
│  ┌──────────┐  ┌──────────┐  ┌──────────────────────┐ │
│  │ 语音监听  │  │ 状态指示灯│  │  Canvas 渲染器        │ │
│  │ (Web      │  │ (录音/    │  │  (Fabric.js)          │ │
│  │  Speech   │  │  思考/    │  │                      │ │
│  │  API)     │  │  执行)    │  │                      │ │
│  └────┬─────┘  └──────────┘  └──────────┬───────────┘ │
│       │                                 │              │
│       │         ┌──────────┐            │              │
│       └────────→│ 指令总线  │←───────────┘              │
│                 │ (Zustand) │                           │
│                 └────┬─────┘                           │
└──────────────────────┼─────────────────────────────────┘
                       │ invoke()
┌──────────────────────┼─────────────────────────────────┐
│                   🦀  Rust 后端层 (Tauri)               │
│                      │                                  │
│  ┌──────────┐  ┌─────┴──────┐  ┌──────────────────┐   │
│  │ 指令解析  │  │ LLM 调度器  │  │  Canvas 命令引擎  │   │
│  │ (预处理/  │  │ (DeepSeek   │  │ (工具函数执行)    │   │
│  │  去噪/    │  │  API 调用)  │  │                  │   │
│  │  纠错)    │  │            │  │  create_node()   │   │
│  └─────┬────┘  └─────┬──────┘  │  add_edge()      │   │
│        │              │         │  auto_layout()   │   │
│        └──────────────┘         │  set_style()     │   │
│               │                 │  undo_last()     │   │
│               └─────────────────┤                  │   │
│                                 └────────┬─────────┘   │
└──────────────────────────────────────────┼─────────────┘
                                           │
                          ┌────────────────┼────────────────┐
                          │      ☁️ 云端 (DeepSeek V4)       │
                          │                                  │
                          │  System Prompt + 工具定义         │
                          │  + 对话历史                       │
                          │  → LLM 决策: 调用哪些工具函数      │
                          │  ← 工具调用结果反馈               │
                          └──────────────────────────────────┘
```

### 2.2 核心设计模式：Function Calling + Canvas 状态机

LLM 不直接生成 Canvas 操作，而是通过 **语义化工具函数** 来操控画布。每次工具调用都是闭环：

```
用户语音 → STT文本 → Rust预处理 → DeepSeek API（含工具定义）
  → LLM 决定调用工具 A → Rust 执行 → 返回结果给 LLM
  → LLM 决定调用工具 B → Rust 执行 → 返回结果给 LLM
  → ... 
  → 最终状态通过 Tauri Event 推送到 React Canvas 渲染
```

**为什么选这个模式：**
- LLM 始终「看到」Canvas 当前状态，决策有依据
- 语义化工具（`add_process_node` 而不是 `draw_rect`）让 LLM 用业务语言思考
- 多轮对话 + 迭代修改天然支持
- 与 Tauri `invoke` 模式完美匹配

### 2.3 一次语音指令的完整旅程

1. 🎤 用户说话 → **Web Speech API** 实时转录为中文 (~200ms)
2. 📝 文本进入 **React Zustand Store**，触发「思考中」状态
3. ⚡ React 通过 `invoke("process_command", {text})` 调用 Rust 后端
4. 🦀 Rust **预处理器**：去噪 → 快捷匹配 → 上下文补全
5. ☁️ Rust **LLM 调度器** 发送 System Prompt + 工具定义 + 对话历史到 DeepSeek
6. 🤖 LLM 返回 **工具调用列表**（如 `add_nodes_batch`, `add_edges_batch`, `auto_layout`）
7. 🦀 Rust **命令引擎** 逐条执行工具调用，结果回传 LLM
8. 🔄 LLM 根据结果可能继续调用更多工具（最多 5 轮循环）
9. ✅ 执行完毕，通过 Tauri Event 推送最终状态到 React
10. 🔊 可选 TTS 语音反馈：「已创建包含 5 个节点的流程图」

---

## 3. 工具函数集设计（15 个）

### 3.1 设计原则

- **语义优先**：用 `add_process_node(label)` 而非 `create_rect(w,h,x,y)`
- **粗粒度**：一个工具调用完成一个完整操作，减少往返
- **状态可见**：每个工具返回操作结果 + 相关上下文

### 3.2 画布管理（3 个）

```rust
create_canvas(title, width?, height?, background?)
  → {canvas_id, width, height}

clear_canvas(canvas_id)
  → {success}  // 需用户确认

get_canvas_state(canvas_id)
  → {nodes: [...], edges: [...], theme, ...}  // 完整快照
```

### 3.3 节点操作（5 个）

```rust
add_node(canvas_id, type, label, position?, style?)
  // type: "start"|"end"|"process"|"decision"|"data"|"subprocess"|"text"
  → {node_id, type, label, x, y, width, height}

add_nodes_batch(canvas_id, nodes: [{type, label, position?}])
  // 推荐 LLM 优先使用此工具，减少往返
  → {nodes: [{node_id, type, label, x, y}], auto_layout: bool}

update_node(node_id, label?, style?, position?)
  → {node_id, changed_fields: [...]}

delete_node(node_id)
  → {success, deleted_edges: [...]}  // 级联删除关联边

move_node(node_id, position | "left" | "right" | "up" | "down")
  → {node_id, old_position, new_position}
```

### 3.4 连线操作（3 个）

```rust
add_edge(from_node_id, to_node_id, label?, style?)
  // style: "solid"|"dashed"|"dotted", arrow: "single"|"double"|"none"
  → {edge_id, from, to, label}

add_edges_batch(canvas_id, edges: [{from, to, label?}])
  → {edges: [{edge_id, from, to}], count: N}

update_edge(edge_id, label?, style?)
  → {edge_id, changed_fields: [...]}
```

### 3.5 样式与布局（4 个）

```rust
set_theme(canvas_id, theme)
  // theme: "default"|"professional"|"handdrawn"|"dark"|"colorful"
  → {theme, changed_count: N}

set_style(target_type, target_id, style)
  // 设置颜色、字体、边框、圆角等
  → {target_type, target_id, applied_style}

auto_layout(canvas_id, direction?)
  // direction: "top_down"|"left_right"|"radial"|"organic"
  → {layout, moved_count: N}

set_zoom_focus(canvas_id, target_type?, target_id?)
  → {zoom_level, center_position}
```

### 3.6 复杂指令拆解示例

用户：「画一个用户登录流程图，包含手机号和验证码登录两种方式，失败重试3次锁定」

LLM 多轮拆解：
1. `add_nodes_batch` — 创建 7 个节点（开始、决策、手机号、验证码、验证判断、成功、结束）
2. `add_edges_batch` — 创建全部连线
3. `add_node` + `add_edges_batch` — 补充重试锁定分支
4. `auto_layout` + `set_theme` — 美化

---

## 4. Rust 后端详细设计

### 4.1 模块一：指令预处理器（CommandPreprocessor）

```
用户文本 → [去噪] → [快捷匹配] → [上下文补全] → 分流到 LLM 或直接执行
```

**去噪（Denoise）：** 移除填充词、重复词、口语噪音。「嗯...那个...画一个...红色圆形」→「画一个红色圆形」

**快捷匹配（QuickMatch）：** 本地指令表，直接执行不调 LLM：

| 用户说 | 执行 |
|--------|------|
| "撤销" / "回退" | `undo_last()` |
| "重做" | `redo()` |
| "清空画布" / "全部删除" | `clear_canvas()` [需确认] |
| "放大" / "缩小" | `zoom_in()` / `zoom_out()` |
| "适应窗口" | `fit_to_screen()` |
| "导出" / "保存" | `export_diagram()` |

**上下文补全（ContextEnrich）：** 代词消解
- 上一轮：「创建一个登录流程图」
- 当前：「把那个菱形改成绿色」
- 补全：「把[登录流程图中的决策节点(node_3)]改成绿色」

### 4.2 模块二：LLM 调度器（LLMScheduler）

```
loop (max 5 rounds):
  ① POST DeepSeek API (streaming)
  ② 如果返回 content → 结束
  ③ 如果返回 tool_calls → 执行工具
  ④ 将 tool_result 追加到 messages
  ⑤ 回到 ①
```

**System Prompt 核心要点：**
> 你是一个语音控制绘图助手。用户通过语音操控图表编辑器。你可以通过工具函数创建流程图、思维导图、架构图等。尽可能一次性批量操作，减少工具调用轮次。布局上优先使用自动布局，不要手动指定每个节点的像素坐标。如果用户说的是修改操作，先理解上下文再精确修改。所有回复使用中文。

### 4.3 模块三：命令引擎（CommandEngine）

**Canvas 状态模型（内存）：**

```rust
struct CanvasState {
    id: String,
    title: String,
    nodes: HashMap<String, DiagramNode>,
    edges: HashMap<String, DiagramEdge>,
    theme: Theme,
    undo_stack: Vec<CanvasSnapshot>,
    redo_stack: Vec<CanvasSnapshot>,
}

struct DiagramNode {
    id: String,
    node_type: NodeType,  // Start, End, Process, Decision, Data, Subprocess, Text
    label: String,
    position: Option<Position>,
    style: NodeStyle,
}

struct DiagramEdge {
    id: String,
    from_id: String,
    to_id: String,
    label: Option<String>,
    line_style: LineStyle,
    arrow: ArrowType,
}
```

- 每次操作前 push undo_stack
- 批量操作（add_nodes_batch）在 Engine 内自动布局
- 执行完成后通过 Tauri Event 推送最终状态到前端

### 4.4 Rust 项目结构

```
src-tauri/src/
├── main.rs                  // Tauri 入口
├── lib.rs                   // 模块注册
├── commands/
│   └── mod.rs               // Tauri #[command] handlers
├── preprocessor/
│   ├── mod.rs               // 预处理入口
│   ├── denoise.rs           // 去噪处理
│   ├── quick_match.rs       // 快捷指令匹配
│   └── context_enrich.rs    // 上下文补全（代词消解）
├── llm/
│   ├── mod.rs               // LLM 调度器入口
│   ├── scheduler.rs         // 主循环 + 工具调用循环
│   ├── system_prompt.rs     // System Prompt 模板
│   ├── tool_defs.rs         // 15 个工具的 JSON Schema 定义
│   └── deepseek_client.rs   // DeepSeek API 客户端 (async-openai)
├── engine/
│   ├── mod.rs               // 命令引擎入口
│   ├── canvas_state.rs      // CanvasState 数据结构
│   ├── node_ops.rs          // 节点 CRUD
│   ├── edge_ops.rs          // 连线 CRUD
│   ├── layout.rs            // 自动布局算法
│   ├── style_ops.rs         // 样式与主题
│   └── snapshot.rs          // undo/redo 快照
└── config/
    ├── mod.rs               // 配置管理
    └── api_key.rs           // API Key 安全存储
```

---

## 5. React 前端详细设计

### 5.1 界面布局

极简三栏设计——Canvas 最大化（90%+），无菜单无按钮：

```
┌─────────────────────────────────────────────────────────┐
│ 🎤 VoiceDraw  [登录流程图]          💡 说出你的想法  │  ← 顶部栏 (32px)
│                                             ⚫ 待机中  │    标题 + 状态灯
├─────────────────────────────────────────────────────────┤
│                                                         │
│                    📐 Canvas 画布区                      │
│                   (Fabric.js 渲染)                       │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  🎤 正在听... ────────────────────────  点按或说"开始"  │  ← 底部语音条 (40px)
│  "画一个登录流程，包含手机号..."                          │     录音波形 + 实时文本
└─────────────────────────────────────────────────────────┘
```

### 5.2 四大核心组件

#### VoiceController（语音控制器）
- 封装 Web Speech API，管理录音生命周期
- **按住说话**（Push-to-Talk）：点按开始录音，松手结束
- **唤醒词**（Wake Word）：说「开始画图」唤醒，持续监听直到静默 2 秒
- 实时转录文本显示在底部语音条
- 录音音量波形可视化（Web Audio API analyser）
- 容错：中文同音词映射、3 次连续低置信度 → 提示重说

#### CanvasRenderer（Canvas 渲染器）
- 将 CanvasState 渲染为 Fabric.js 对象
- 节点映射：Start/End→圆角矩形，Process→矩形，Decision→菱形，Data→平行四边形
- 4 套主题：「专业」「手绘」「暗色」「多彩」
- 入场动画：节点 scale 0→1 弹性动画 (300ms)，自动布局过渡 (500ms)

#### StatusIndicator（状态指示灯）
- 5 种状态：⚪待机 🟡监听中 🔵思考中 🟢执行中 🔴出错
- Canvas 浮层 Toast：「✅ 已创建 5 个节点」「🔄 正在重新布局」「❌ 操作失败，请重试」

#### CommandStore（Zustand 状态管理）

```typescript
interface AppState {
  isListening: boolean
  transcript: string
  status: 'idle' | 'listening' | 'thinking' | 'executing' | 'error'
  canvasState: CanvasState | null
  lastOperation: string
  conversation: Message[]
  
  startListening: () => void
  submitCommand: (text: string) => Promise<void>
  updateCanvasState: (state: CanvasState) => void
  quickAction: (action: string) => void
}
```

单向数据流：`VoiceController → Store.submitCommand → invoke() → Rust → Event → Store.updateCanvasState → Canvas 重渲染`

### 5.3 React 组件树

```
src/
├── main.tsx
├── App.tsx
├── components/
│   ├── layout/
│   │   ├── TopBar.tsx          // 标题栏 + 状态灯
│   │   └── VoiceBar.tsx        // 底部语音控制条
│   ├── canvas/
│   │   ├── CanvasView.tsx      // Fabric.js 容器
│   │   ├── nodes/              // 各类型节点渲染器
│   │   ├── edges/              // 连线渲染
│   │   └── CanvasOverlay.tsx   // 提示浮层
│   ├── voice/
│   │   ├── VoiceController.tsx // Web Speech 封装
│   │   └── WaveformVisualizer.tsx
│   └── status/
│       ├── StatusLight.tsx
│       └── Toast.tsx
├── store/
│   ├── index.ts                // Zustand store
│   └── types.ts
├── hooks/
│   ├── useVoiceRecognition.ts
│   ├── useCanvasBridge.ts
│   └── useKeyboardShortcut.ts  // 开发者调试用（可选）
└── lib/
    ├── fabric-setup.ts
    └── theme-presets.ts
```

---

## 6. 容错与纠错设计

### 6.1 四层防线模型

| 层 | 位置 | 典型问题 | 策略 |
|----|------|---------|------|
| 第 1 层 | STT | 识别错误（「话圆」→「画圆」） | interim 实时显示、同音词映射表、低置信度提示重说 |
| 第 2 层 | Rust 预处理 | 口语噪音、歧义代词、非法指令 | 去噪、代词消解、领域词典、非法检测 |
| 第 3 层 | DeepSeek LLM | 理解偏差、参数错误、幻觉 | enum 约束参数、node_id 不存在时 Engine 报错→LLM 修正、不确定时反问 |
| 第 4 层 | Engine 执行 | 非法操作、状态冲突 | 操作前验证、Undo 栈回滚、批量操作部分成功 |

### 6.2 典型失败场景处理

| 场景 | 用户感知 | 系统行为 |
|------|---------|---------|
| STT 识别乱码 | 底部显示乱码文本 | 用户松手重说，或说「取消」放弃 |
| 网络断开 | 状态灯红 + 「网络异常」 | 快捷指令（撤销/缩放/导出）仍可用 |
| LLM 超时 (>10s) | 「思考超时，请简化指令」 | 取消请求，保留已有操作 |
| LLM 无效工具调用 | 静默处理，灯短暂变红 | 重试 1 次，失败返回部分结果 |
| 用户连续快速说话 | 指令排队等待 | 串行执行，说「取消」清空队列 |
| Canvas 渲染异常 | 画布显示异常 | Fabric.js 错误边界 + 从 State 重建 |
| 用户对结果不满意 | 说「撤销」回退 | Undo 栈回退整个指令批次 |

### 6.3 容错 UX 原则

- **永远给反馈**：纯语音交互中沉默=死亡，任何操作都有状态变化
- **可恢复优先**：能部分成功就部分成功，Undo 随时可用
- **降级不崩溃**：LLM 不可用时本地快捷指令仍能工作
- **用语音解释**：复杂错误用 TTS 语音读出建议

---

## 7. 性能与延迟优化

### 7.1 延迟预算

| 环节 | 耗时 | 优化手段 |
|------|------|---------|
| ① STT 语音识别 | 100-300ms | interim results 实时显示（用户感知 0ms） |
| ② 前端→Rust invoke | ~5ms | IPC 几乎无感 |
| ③ 预处理 | ~5ms | 纯内存操作 |
| ④ DeepSeek 首 token | 300-800ms | stream: true |
| ⑤ LLM 完整生成 | 500-1500ms | 限制 max_tokens，控制工具数量 |
| ⑥ 工具调用执行（每轮） | ~10ms | 纯内存 |
| ⑦ Rust→React Event | ~8ms | Tauri Event |
| ⑧ Fabric.js 渲染 | ~16ms | 60fps，batch 更新 |

### 7.2 六大优化策略

**策略 1：三级指令分流**

| 级别 | 指令类型 | 处理方式 | 延迟 |
|------|---------|---------|------|
| L0 本地 | 撤销/重做/缩放/导出/清空 | 快捷匹配→直接执行 | <50ms |
| L1 简单 | 属性修改/单步操作 | 模板匹配→单轮 LLM | ~1s |
| L2 复杂 | 创建图表/批量修改/布局 | 完整 LLM 多轮工具调用 | 2-4s |

**策略 2：乐观 UI 更新** — L2 指令提交后立即显示占位元素，LLM 每返回一轮工具结果增量更新 Canvas

**策略 3：流式工具调用** — Stream: true，拿到一个 tool_call 立即执行，不需要等全部

**策略 4：批量操作优先** — System Prompt 引导 LLM 使用 batch 工具，一次创建所有节点/连线

**策略 5：对话上下文压缩** — 保留最近 5 轮完整对话，更早的只保留摘要，减少 prompt tokens

**策略 6：语音交互体验** — interim results + TTS 反馈 + 可打断 + 背景噪音自动暂停

### 7.3 性能基准目标

| 场景 | P50 | P95 | 用户感知 |
|------|-----|-----|---------|
| 「撤销」 | 30ms | 50ms | 瞬间 |
| 「把这个改成绿色」 | 800ms | 1.5s | 一眨眼 |
| 「画一个简单流程图 A→B→C」 | 1.5s | 3s | 可接受 |
| 「画一个登录流程图含验证码和重试」 | 3s | 5s | TTS 反馈中等待 |

---

## 8. 技术选型汇总

| 层 | 技术 | 选择理由 |
|----|------|---------|
| 桌面框架 | Tauri 2.x | 轻量（<10MB）、Rust 后端、跨平台 |
| 前端框架 | React 18 + TypeScript | 生态成熟、组件化 |
| Canvas 库 | Fabric.js 6.x | 对象模型清晰，适合形状操作，交互成熟 |
| 状态管理 | Zustand | 轻量、TS 友好、与 Tauri 事件系统适配好 |
| STT (MVP) | Web Speech API | 零依赖，快速验证 |
| STT (最终) | Web Speech API + Whisper.cpp | 本地兜底，离线可用 |
| LLM SDK | async-openai (Rust) | DeepSeek 兼容 OpenAI API |
| LLM | DeepSeek V4 | 性价比高，function calling 能力强 |
| TTS（可选） | Web Speech Synthesis | 零依赖语音反馈 |
| 自动布局 | dagre.js (前端) 或自研 (Rust) | 分层/力导向布局 |

---

## 9. 安全注意事项

- DeepSeek API Key 存储在 Tauri 本地安全存储，不硬编码
- 所有 LLM 调用在 Rust 后端完成，前端不直接接触 API Key
- 用户数据（图表内容、对话历史）仅保存在本地，不上传（除 LLM API 调用外）
- `.superpowers/` 加入 `.gitignore`

---

## 10. 未来扩展方向

- 本地 Whisper.cpp 集成（离线 STT）
- 手绘风格渲染增强
- 导出格式扩展（SVG/PDF/PPTX）
- 多人语音协作绘图
- 自定义图表模板库
- 语音命令宏录制
