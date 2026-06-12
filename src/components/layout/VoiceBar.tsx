import { useAppStore } from "../../store";
import { useVoiceRecognition } from "../../hooks/useVoiceRecognition";

/**
 * 底部语音控制栏
 * 使用 Web Speech API 进行语音识别
 */
export default function VoiceBar() {
  const isListening = useAppStore((s) => s.isListening);
  const transcript = useAppStore((s) => s.transcript);
  const status = useAppStore((s) => s.status);
  const setTranscript = useAppStore((s) => s.setTranscript);
  const setStatus = useAppStore((s) => s.setStatus);
  const quickAction = useAppStore((s) => s.quickAction);

  const { start, stop } = useVoiceRecognition({
    onResult: (text) => {
      setTranscript(text);
    },
    onError: (error) => {
      console.error(error);
      setTranscript("");
      setStatus("error");
      setTimeout(() => setStatus("idle"), 2000);
    },
    onEnd: () => {
      const state = useAppStore.getState();
      const finalText = state.transcript.trim();
      if (finalText.length > 0) {
        state.submitCommand(finalText);
      }
      // 不在 onEnd 中重置 isListening，因为 submitCommand 内部会改 status
    },
  });

  const handleToggle = () => {
    if (isListening) {
      stop();
      useAppStore.setState({ isListening: false });
    } else {
      setTranscript("");
      useAppStore.setState({ isListening: true, status: "listening" });
      start();
    }
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
      minWidth: 520,
    }}>
      <button
        onClick={handleToggle}
        disabled={status !== "idle" && status !== "listening"}
        style={{
          width: 40,
          height: 40,
          borderRadius: "50%",
          border: "none",
          background: isListening ? "#ea4335" : "#f1f3f4",
          color: isListening ? "#fff" : "#5f6368",
          fontSize: 18,
          cursor: "pointer",
          transition: "all 0.2s",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          flexShrink: 0,
        }}
      >
        🎤
      </button>

      <div style={{
        flex: 1,
        height: 36,
        lineHeight: "36px",
        padding: "0 14px",
        borderRadius: 20,
        background: "#f1f3f4",
        color: isListening ? "#1f2328" : "#9aa0a6",
        fontSize: 14,
        overflow: "hidden",
        whiteSpace: "nowrap",
        textOverflow: "ellipsis",
      }}>
        {transcript || (isListening ? "请说话..." : "点击麦克风开始语音指令")}
      </div>

      <button
        onClick={() => quickAction("undo")}
        disabled={status !== "idle"}
        style={{
          padding: "6px 14px",
          borderRadius: 20,
          border: "1px solid #e8eaed",
          background: "#fff",
          color: "#5f6368",
          cursor: "pointer",
          fontSize: 12,
          fontWeight: 500,
          whiteSpace: "nowrap",
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
          color: "#5f6368",
          cursor: "pointer",
          fontSize: 12,
          fontWeight: 500,
          whiteSpace: "nowrap",
        }}
      >
        ↪ 重做
      </button>
    </footer>
  );
}
