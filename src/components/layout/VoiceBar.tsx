import { useCallback, useRef } from "react";
import { useAppStore } from "../../store";
import { useVoiceRecognition } from "../../hooks/useVoiceRecognition";
import WaveformVisualizer from "../voice/WaveformVisualizer";

const STATUS_TEXT: Record<string, string> = {
  idle: "👆 按住说话，或说「开始画图」唤醒",
  listening: "🎤 正在听...",
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
  const quickAction = useAppStore((s) => s.quickAction);

  const silenceTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const lastTextRef = useRef("");

  const handleResult = useCallback((text: string, _isFinal: boolean) => {
    setTranscript(text);
    lastTextRef.current = text;

    // 2 秒静默后自动提交
    if (silenceTimerRef.current) clearTimeout(silenceTimerRef.current);
    silenceTimerRef.current = setTimeout(() => {
      if (lastTextRef.current.trim().length > 0) {
        stop();
        useAppStore.setState({ isListening: false });
      }
    }, 2000);
  }, [setTranscript]);

  const handleError = useCallback((error: string) => {
    console.error("STT Error:", error);
    setStatus("error");
    setTimeout(() => setStatus("idle"), 2000);
  }, [setStatus]);

  const handleEnd = useCallback(() => {
    // recognition 自然结束
  }, []);

  const { start, stop } = useVoiceRecognition({
    onResult: handleResult,
    onError: handleError,
    onEnd: handleEnd,
  });

  // 按住说话
  const handlePointerDown = () => {
    if (status !== "idle" && status !== "listening") return;
    setTranscript("");
    useAppStore.setState({ isListening: true, status: "listening" });
    start();
  };

  const handlePointerUp = () => {
    if (silenceTimerRef.current) clearTimeout(silenceTimerRef.current);
    stop();
    useAppStore.setState({ isListening: false });
  };

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

      {/* 语音输入区域 — 按住说话 */}
      <div
        onPointerDown={handlePointerDown}
        onPointerUp={handlePointerUp}
        onPointerLeave={handlePointerUp}
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
          cursor: "pointer",
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
