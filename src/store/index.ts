import { create } from "zustand";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { CanvasState, AppStatus, ConversationMessage, OperationResult } from "./types";

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
      if (result.success && result.canvas_state) {
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

  setStatus: (status) => set({ status }),

  _updateCanvasState: (state) => set({ canvasState: state }),

  _initEventListener: async () => {
    // 监听 Rust 端推送的 canvas 更新事件
    await listen<CanvasState>("canvas-updated", (event) => {
      set({ canvasState: event.payload, status: "idle" });
    });
  },
}));