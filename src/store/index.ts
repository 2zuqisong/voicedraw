import { create } from "zustand";
import { invoke, listen } from "../lib/apiBridge";
import type { CanvasState, AppStatus, ConversationMessage, OperationResult, OperationPlan, PendingAction, CanvasMode, PixelTool, PixelState } from "./types";
import { getLLMApiKey, getLLMEndpoint, getLLMModel } from "../lib/settings";

interface AppState {
  // 语音状态
  isListening: boolean;
  transcript: string;
  status: AppStatus;

  // Canvas 状态
  canvasState: CanvasState | null;
  lastOperation: string;

  // 对话历史
  conversation: ConversationMessage[];

  // 操作计划
  pendingPlan: OperationPlan | null;
  confirmPlan: () => Promise<void>;
  cancelPlan: () => Promise<void>;
  modifyPlan: (newText: string) => Promise<void>;

  // 待前端执行的异步操作（如风格转换）
  pendingAction: PendingAction | null;
  clearPendingAction: () => void;

  // 导出信号
  exportFormat: string | null;
  clearExport: () => void;

  // 动作
  startListening: () => void;
  stopListening: () => void;
  setTranscript: (text: string) => void;
  submitCommand: (text: string) => Promise<void>;
  quickAction: (action: string) => Promise<void>;
  setStatus: (status: AppStatus) => void;

  // Undo/Redo 可用性
  canUndo: boolean;
  canRedo: boolean;
  _refreshSnapshotStatus: () => Promise<void>;

  // ── 像素模式 ──
  canvasMode: CanvasMode;
  pixel: PixelState;
  setCanvasMode: (mode: CanvasMode) => void;
  _syncPixelFromCanvas: () => void;
  setPixelCell: (row: number, col: number, color: string | null) => void;
  setPixelColor: (color: string) => void;
  setPixelTool: (tool: PixelTool) => void;
  setPixelCellSize: (size: number) => void;
  pixelFloodFill: (row: number, col: number, color: string) => void;
  clearPixelData: () => void;
  pixelUndo: () => void;

  // 内部
  _updateCanvasState: (state: CanvasState) => void;
  _initEventListener: () => Promise<void>;
}

const DEFAULT_PIXEL_COLS = 32;
const DEFAULT_PIXEL_ROWS = 32;
const DEFAULT_PIXEL_CELL = 20;

function createDefaultPixel(): PixelState {
  return {
    data: {},
    color: "#1a1a1a",
    tool: "pencil",
    cellSize: DEFAULT_PIXEL_CELL,
    cols: DEFAULT_PIXEL_COLS,
    rows: DEFAULT_PIXEL_ROWS,
    undoStack: [],
  };
}

function floodFill(
  data: Record<string, string>,
  cols: number,
  rows: number,
  startRow: number,
  startCol: number,
  fillColor: string,
): Record<string, string> {
  const key = `${startRow},${startCol}`;
  const targetColor = data[key] ?? null;

  // 如果目标颜色和填充颜色相同，无需操作
  if (targetColor === fillColor) return data;

  const result = { ...data };
  const stack: [number, number][] = [[startRow, startCol]];
  const visited = new Set<string>();
  visited.add(key);

  while (stack.length > 0) {
    const [r, c] = stack.pop()!;
    const k = `${r},${c}`;
    const current = result[k] ?? null;

    if (current === targetColor) {
      if (fillColor === "" || fillColor === null) {
        delete result[k];
      } else {
        result[k] = fillColor;
      }

      // 检查四邻域
      for (const [dr, dc] of [[-1, 0], [1, 0], [0, -1], [0, 1]]) {
        const nr = r + dr;
        const nc = c + dc;
        if (nr >= 0 && nr < rows && nc >= 0 && nc < cols) {
          const nk = `${nr},${nc}`;
          if (!visited.has(nk)) {
            visited.add(nk);
            stack.push([nr, nc]);
          }
        }
      }
    }
  }

  return result;
}

export const useAppStore = create<AppState>((set, get) => ({
  isListening: false,
  transcript: "",
  status: "idle",
  canvasState: null,
  lastOperation: "",
  conversation: [],
  pendingPlan: null,
  pendingAction: null,
  clearPendingAction: () => set({ pendingAction: null }),
  exportFormat: null,
  clearExport: () => set({ exportFormat: null }),
  canUndo: false,
  canRedo: false,

  // ── 像素模式 ──
  canvasMode: "vector",
  pixel: createDefaultPixel(),
  setCanvasMode: (mode) => set({ canvasMode: mode }),

  /** 从 canvasState.pixel 同步到本地 Zustand（LLM 修改后触发） */
  _syncPixelFromCanvas: () => {
    const cs = get().canvasState;
    if (!cs?.pixel) return;
    const prev = get().pixel;
    set({
      pixel: {
        ...prev,
        data: { ...cs.pixel.cells },
        cols: cs.pixel.cols,
        rows: cs.pixel.rows,
        cellSize: cs.pixel.cell_size,
      },
    });
  },

  setPixelCell: (row, col, color) => {
    const { pixel } = get();
    const key = `${row},${col}`;
    const prevData = { ...pixel.data };
    const newData = { ...pixel.data };
    if (color === null || color === "") {
      delete newData[key];
    } else {
      newData[key] = color;
    }
    const undoStack = [...pixel.undoStack, prevData].slice(-30);
    set({ pixel: { ...pixel, data: newData, undoStack } });
  },

  setPixelColor: (color) => {
    const { pixel } = get();
    set({ pixel: { ...pixel, color } });
  },

  setPixelTool: (tool) => {
    const { pixel } = get();
    set({ pixel: { ...pixel, tool } });
  },

  setPixelCellSize: (size) => {
    const { pixel } = get();
    // 调整网格大小：保持画布尺寸一致
    const canvasW = pixel.cols * pixel.cellSize;
    const cols = Math.round(canvasW / size);
    const rows = Math.round(canvasW / size);
    set({ pixel: { ...pixel, cellSize: size, cols, rows, undoStack: [] } });
  },

  pixelFloodFill: (row, col, color) => {
    const { pixel } = get();
    const prevData = { ...pixel.data };
    const newData = floodFill(pixel.data, pixel.cols, pixel.rows, row, col, color);
    const undoStack = [...pixel.undoStack, prevData].slice(-30);
    set({ pixel: { ...pixel, data: newData, undoStack } });
  },

  clearPixelData: () => {
    const { pixel } = get();
    const prevData = { ...pixel.data };
    const undoStack = [...pixel.undoStack, prevData].slice(-30);
    set({ pixel: { ...pixel, data: {}, undoStack } });
  },

  pixelUndo: () => {
    const { pixel } = get();
    const stack = [...pixel.undoStack];
    if (stack.length === 0) return;
    const prevData = stack.pop()!;
    set({ pixel: { ...pixel, data: prevData, undoStack: stack } });
  },

  _refreshSnapshotStatus: async () => {
    try {
      const status = await invoke<{ can_undo: boolean; can_redo: boolean }>("get_snapshot_status");
      set({ canUndo: status.can_undo, canRedo: status.can_redo });
    } catch { /* 忽略 */ }
  },

  startListening: () => set({ isListening: true, status: "listening", transcript: "" }),
  
  stopListening: () => {
    const text = get().transcript.trim();
    set({ isListening: false });
    if (text.length > 0) {
      get().submitCommand(text);
    } else {
      set({ status: "idle" });
    }
  },

  setTranscript: (text) => set({ transcript: text }),

  submitCommand: async (text) => {
    set({ status: "thinking", lastOperation: text });
    try {
      const result = await invoke<OperationResult>("process_command", {
        text,
        llmApiKey: getLLMApiKey(),
        llmEndpoint: getLLMEndpoint(),
        llmModel: getLLMModel(),
      });
      // 检测导出快捷操作
      if (result.message && result.message.includes("快捷操作: export")) {
        const fmt = text.includes("SVG") || text.includes("svg") ? "svg" : "png";
        set({ exportFormat: fmt, status: "idle", lastOperation: `正在导出 ${fmt.toUpperCase()}...` });
        return;
      }
      if (result.pending_plan) {
        // 复杂指令：显示预览面板
        set({
          pendingPlan: result.pending_plan,
          status: "idle",
          lastOperation: result.message,
        });
      } else if (result.pending_action) {
        // 异步操作（如风格转换）：存储 pendingAction，由 CanvasView 检测并执行
        set({
          pendingAction: result.pending_action,
          status: "executing",
          lastOperation: result.message,
          conversation: [
            ...get().conversation,
            { role: "user" as const, content: text },
            { role: "assistant" as const, content: result.message },
          ].slice(-10),
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
          ].slice(-10), // 保留最近10条
        });
        get()._refreshSnapshotStatus();
        get()._syncPixelFromCanvas();
      } else {
        set({ status: "error", lastOperation: result.message || "操作失败" });
      }
    } catch (err) {
      const errorMsg = err instanceof Error ? err.message : String(err);
      set({ status: "error", lastOperation: `错误: ${errorMsg}` });
      // 3 秒后自动恢复
      setTimeout(() => set({ status: "idle" }), 3000);
    }
  },

  quickAction: async (action) => {
    set({ status: "executing", lastOperation: action });
    try {
      const result = await invoke<OperationResult>("quick_action", { action });
      if (result.success && result.canvas_state) {
        set({
          canvasState: result.canvas_state,
          status: "idle",
          lastOperation: result.message,
        });
        get()._syncPixelFromCanvas();
      } else {
        set({ status: "error", lastOperation: result.message });
        setTimeout(() => set({ status: "idle" }), 2000);
      }
    } catch (err) {
      set({ status: "error", lastOperation: String(err) });
      setTimeout(() => set({ status: "idle" }), 2000);
    }
  },

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
        get()._syncPixelFromCanvas();
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
        get()._syncPixelFromCanvas();
      } else {
        set({ status: "error", pendingPlan: null, lastOperation: result.message || "修改失败" });
        setTimeout(() => set({ status: "idle" }), 3000);
      }
    } catch (err) {
      set({ status: "error", pendingPlan: null, lastOperation: String(err) });
      setTimeout(() => set({ status: "idle" }), 3000);
    }
  },

  setStatus: (status) => set({ status }),

  _updateCanvasState: (state) => {
    set({ canvasState: state });
    get()._syncPixelFromCanvas();
  },

  _initEventListener: async () => {
    // 监听 Rust 端推送的 canvas 更新事件
    await listen<CanvasState>("canvas-updated", (event) => {
      set({ canvasState: event.payload, status: "idle", pendingPlan: null });
      get()._refreshSnapshotStatus();
      get()._syncPixelFromCanvas();
    });
  },
}));