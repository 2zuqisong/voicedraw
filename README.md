# voice-draw

> 中文语音 → AI 理解 → 画布绘图

[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/tauri-2.x-blue)](https://tauri.app/)
[![React](https://img.shields.io/badge/react-19.x-61dafb)](https://react.dev/)

说话就能画画。AI 理解中文语音指令，在画布上实时生成矢量图形和像素表情。支持流程图、几何图形、风格转换、模板一键生成。

---

## 功能

**矢量模式** — 流程图 · 架构图 · 20 种几何图形 · 5 个预设模板 · 10 种风格转换 · PNG/SVG 导出

**像素模式** — 32×32 网格 · 8 种聊天表情小人 · 纯 LLM 驱动绘制

**语音** — Web Speech API 连续收音 · 文本输入降级 · 多厂商 LLM 可切换

## 快速开始

```bash
# 前提：Bun / Rust 1.80+ / Linux 需 libwebkit2gtk
git clone https://github.com/2zuqisong/voicedraw.git
cd voicedraw
bun install
bun run tauri dev
```

浏览器打开 `http://localhost:1420/`（桌面窗口默认隐藏）。

右上角 ⚙ 配置 LLM API Key，或设环境变量 `DEEPSEEK_API_KEY`。

## 使用

### 矢量模式

| 说 | 画 |
|----|-----|
| "画登录流程：开始→输入→验证→成功" | 流程图 + 自动布局 |
| "画一座蓝色房子，左边一棵树" | 复合图形 + 相对定位 |
| "画生日贺卡" | 一键模板 |
| "变成梵高风格" | 全画布油画风格转换 |
| "导出 PNG" | 下载图片 |

### 像素模式

点顶部「像素」切换：

| 说 | 画 |
|----|-----|
| "画一个笑脸" | 😊 |
| "左边笑哭，右边生气" | 并排两个 |
| "四个表情：爱心眼、酷、笑脸、惊讶" | 四宫格 |
| "把背景涂成蓝色" | 区域填充 |

## 技术栈

| 层 | 选型 |
|----|------|
| 桌面框架 | Tauri 2.x |
| 前端 | React 19 + TypeScript + Vite |
| 画布 | Fabric.js v6（矢量）· Canvas 2D（像素） |
| 后端 | Rust · serde · tokio · async-openai · axum |
| LLM | DeepSeek / OpenAI 兼容 API（tool-calling） |
| 语音 | Web Speech API |
| 布局 | dagre（自动排列） |

## 架构

```
浏览器 → HTTP API (:1421, axum) → Rust Engine
 Tauri WebView → IPC invoke ─────→ 共享同一 Engine
                                       │
                          LLM Scheduler ← DeepSeek/OpenAI
                          CanvasState (nodes/edges/pixel)
风格转换 → DashScope 通义万相
```

## 目录

```
src/                     # React 前端
├── components/canvas/    # CanvasView / PixelCanvas / ShapeRenderer
├── components/layout/    # TopBar / VoiceBar / ShapePalette
├── hooks/                # useVoiceRecognition
├── lib/                  # apiBridge / fabric-setup / settings
└── store/                # Zustand

src-tauri/src/           # Rust 后端
├── commands/             # Tauri 命令
├── engine/               # 绘图引擎 (shapes/emoji/templates/layout/style)
├── llm/                  # LLM 调度 (scheduler/tools/prompts)
├── preprocessor/         # 文本预处理
└── http_server.rs        # HTTP API
```
