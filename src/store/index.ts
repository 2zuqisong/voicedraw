import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { CanvasState, AppStatus, ConversationMessage, OperationResult, OperationPlan } from "./types";

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

  // 动作
  startListening: () => void;
  stopListening: () => void;
  setTranscript: (text: string) => void;
  submitCommand: (text: string) => Promise<void>;
  quickAction: (action: string) => Promise<void>;
  setStatus: (status: AppStatus) => void;

  // 内部
  _updateCanvasState: (state: CanvasState) => void;
  _initEventListener: () => Promise<void>;
}

export const useAppStore = create<AppState>((set, get) => ({
  isListening: false,
  transcript: "",
  status: "idle",
  canvasState: null,
  lastOperation: "",
  conversation: [],
  pendingPlan: null,

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
          ].slice(-10), // 保留最近10条
        });
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

  setStatus: (status) => set({ status }),

  _updateCanvasState: (state) => set({ canvasState: state }),

  _initEventListener: async () => {
    // 监听 Rust 端推送的 canvas 更新事件
    await listen<CanvasState>("canvas-updated", (event) => {
      set({ canvasState: event.payload, status: "idle" });
    });
  },
}));