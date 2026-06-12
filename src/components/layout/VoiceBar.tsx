import { useCallback, useRef } from "react";
import { useAppStore } from "../../store";
import { useVoiceRecognition } from "../../hooks/useVoiceRecognition";
import WaveformVisualizer from "../voice/WaveformVisualizer";

const STATUS_TEXT: Record<string, string> = {
  idle: "点击麦克风开始语音指令",
  listening: "正在聆听...",
  thinking: "🤔 正在理解...",
  executing: "✏️ 正在绘图...",
  error: "❌ 出错了，点击重试",
};

export default function VoiceBar() {
  const isListening = useAppStore((s) => s.isListening);
  const transcript = useAppStore((s) => s.transcript);
  const status = useAppStore((s) => s.status);
  const setTranscript = useAppStore((s) => s.setTranscript);
  const setStatus = useAppStore((s) => s.setStatus);
  const submitCommand = useAppStore((s) => s.submitCommand);
  const quickAction = useAppStore((s) => s.quickAction);

  const lastTextRef = useRef("");

  const handleResult = useCallback((text: string, _isFinal: boolean) => {
    setTranscript(text);
    lastTextRef.current = text;
  }, [setTranscript]);

  const handleError = useCallback((error: string) => {
    console.error("STT Error:", error);
    setStatus("error");
    setTimeout(() => setStatus("idle"), 2000);
  }, [setStatus]);

  const handleEnd = useCallback(() => {
    // recognition 自然结束
  }, []);

  const { start, stop, abort } = useVoiceRecognition({
    onResult: handleResult,
    onError: handleError,
    onEnd: handleEnd,
  });

  // 开始录音
  const handleStart = () => {
    if (status !== "idle") return;
    setTranscript("");
    useAppStore.setState({ isListening: true, status: "listening" });
    start();
  };

  // 确认提交
  const handleConfirm = () => {
    stop();
    useAppStore.setState({ isListening: false });
    const text = lastTextRef.current.trim();
    if (text.length > 0) {
      submitCommand(text);
    }
  };

  // 取消录音
  const handleCancel = () => {
    abort();
    setTranscript("");
    useAppStore.setState({ isListening: false, status: "idle" });
  };

  const isBusy = status !== "idle" && status !== "listening";

  return (
    <footer className="voice-bar" style={{
      position: "absolute",
      bottom: 16,
      left: "50%",
      transform: "translateX(-50%)",
      display: "flex",
      alignItems: "center",
      gap: 10,
      padding: "8px 14px",
      background: "#ffffff",
      borderRadius: 40,
      boxShadow: "0 2px 16px rgba(0,0,0,0.10), 0 1px 4px rgba(0,0,0,0.06)",
      border: "1px solid #e8eaed",
      zIndex: 100,
      minWidth: 560,
      userSelect: "none",
    }}>
      {/* 波形可视化 */}
      <WaveformVisualizer isActive={isListening} />

      {/* 取消按钮 — 仅在录音中显示 */}
      {isListening && (
        <button
          onClick={handleCancel}
          style={{
            width: 36,
            height: 36,
            borderRadius: "50%",
            border: "none",
            background: "#ea4335",
            color: "#fff",
            fontSize: 18,
            cursor: "pointer",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            flexShrink: 0,
            transition: "all 0.2s",
          }}
          title="取消"
        >
          ✕
        </button>
      )}

      {/* 语音输入区域 */}
      <div
        onClick={isListening ? undefined : handleStart}
        style={{
          flex: 1,
          height: 36,
          borderRadius: 20,
          background: isListening
            ? "linear-gradient(135deg, #667eea 0%, #764ba2 100%)"
            : "#f1f3f4",
          border: isListening ? "2px solid #667eea" : "2px solid transparent",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          cursor: isBusy ? "default" : "pointer",
          transition: "all 0.2s",
          padding: "0 14px",
          overflow: "hidden",
        }}
      >
        <span style={{
          color: isListening ? "#fff" : "#9aa0a6",
          fontSize: 13,
          whiteSpace: "nowrap",
          textOverflow: "ellipsis",
          overflow: "hidden",
        }}>
          {isListening
            ? (transcript || "正在听...")
            : STATUS_TEXT[status] || STATUS_TEXT.idle}
        </span>
      </div>

      {/* 确认按钮 — 仅在录音中且有内容时显示 */}
      {isListening && (
        <button
          onClick={handleConfirm}
          style={{
            width: 36,
            height: 36,
            borderRadius: "50%",
            border: "none",
            background: transcript.trim().length > 0 ? "#34a853" : "#dadce0",
            color: transcript.trim().length > 0 ? "#fff" : "#9aa0a6",
            fontSize: 18,
            cursor: transcript.trim().length > 0 ? "pointer" : "default",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            flexShrink: 0,
            transition: "all 0.2s",
          }}
          title="确认"
        >
          ✓
        </button>
      )}

      {/* 字数统计 */}
      <span style={{
        fontSize: 11,
        color: "#9aa0a6",
        minWidth: 36,
        textAlign: "right",
        flexShrink: 0,
      }}>
        {transcript.length > 0 ? `${transcript.length} 字` : ""}
      </span>

      {/* 撤销 / 重做 */}
      <button
        onClick={() => quickAction("undo")}
        disabled={status !== "idle"}
        style={{
          padding: "6px 14px",
          borderRadius: 20,
          border: "1px solid #e8eaed",
          background: "#fff",
          color: status !== "idle" ? "#ccc" : "#5f6368",
          cursor: status !== "idle" ? "default" : "pointer",
          fontSize: 12,
          fontWeight: 500,
          whiteSpace: "nowrap",
          flexShrink: 0,
        }}
      >
        ↩ 撤销
      </button>
      <button
        onClick={() => quickAction("redo")}
        disabled={status !== "idle"}
        style={{
          padding: "6px 14px",
          borderRadius: 20,
          border: "1px solid #e8eaed",
          background: "#fff",
          color: status !== "idle" ? "#ccc" : "#5f6368",
          cursor: status !== "idle" ? "default" : "pointer",
          fontSize: 12,
          fontWeight: 500,
          whiteSpace: "nowrap",
          flexShrink: 0,
        }}
      >
        ↪ 重做
      </button>
    </footer>
  );
}
