import { useEffect, useRef } from "react";
import { useAppStore } from "../../store";

/**
 * 底部语音控制栏
 * 使用 Web Speech API 进行语音识别
 */
export default function VoiceBar() {
  const isListening = useAppStore((s) => s.isListening);
  const transcript = useAppStore((s) => s.transcript);
  const status = useAppStore((s) => s.status);
  const startListening = useAppStore((s) => s.startListening);
  const stopListening = useAppStore((s) => s.stopListening);
  const setTranscript = useAppStore((s) => s.setTranscript);
  const quickAction = useAppStore((s) => s.quickAction);
  const recognitionRef = useRef<SpeechRecognition | null>(null);

  useEffect(() => {
    const SpeechRecognition = window.SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (!SpeechRecognition) {
      console.warn("Web Speech API 不可用");
      return;
    }

    const recognition = new SpeechRecognition();
    recognition.lang = "zh-CN";
    recognition.interimResults = true;
    recognition.maxAlternatives = 2;

    recognition.onresult = (event: SpeechRecognitionEvent) => {
      let interim = "";
      let final = "";
      for (let i = event.resultIndex; i < event.results.length; i++) {
        const result = event.results[i];
        if (result.isFinal) {
          final += result[0].transcript;
        } else {
          interim += result[0].transcript;
        }
      }
      setTranscript(final + interim);
    };

    recognition.onerror = (event: any) => {
      console.error("语音识别错误:", event.error);
      stopListening();
    };

    recognition.onend = () => {
      // 如果用户仍然在聆听状态，自动重启
      if (useAppStore.getState().isListening) {
        recognition.start();
      } else {
        // 识别结束后提交
        const text = useAppStore.getState().transcript.trim();
        if (text.length > 0) {
          useAppStore.getState().submitCommand(text);
        }
      }
    };

    recognitionRef.current = recognition;
    return () => { recognition.stop(); };
  }, []);

  // 控制识别启停
  useEffect(() => {
    const rec = recognitionRef.current;
    if (!rec) return;
    try {
      if (isListening) {
        rec.start();
      } else {
        rec.stop();
      }
    } catch {
      // 忽略重复 start/stop 的错误
    }
  }, [isListening]);

  const handleToggle = () => {
    if (isListening) {
      stopListening();
    } else {
      startListening();
    }
  };

  return (
    <footer className="voice-bar" style={{
      height: 72,
      display: "flex",
      alignItems: "center",
      gap: 12,
      padding: "0 16px",
      background: "#16213e",
      borderTop: "1px solid #0f3460",
      flexShrink: 0,
    }}>
      <button
        onClick={handleToggle}
        disabled={status !== "idle" && status !== "listening"}
        style={{
          width: 44,
          height: 44,
          borderRadius: "50%",
          border: "none",
          background: isListening ? "#e94560" : "#0f3460",
          color: "#fff",
          fontSize: 20,
          cursor: "pointer",
          transition: "background 0.2s",
        }}
      >
        🎤
      </button>

      <div style={{
        flex: 1,
        height: 36,
        lineHeight: "36px",
        padding: "0 12px",
        borderRadius: 8,
        background: "#0f3460",
        color: isListening ? "#fff" : "#888",
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
          padding: "6px 12px",
          borderRadius: 6,
          border: "1px solid #0f3460",
          background: "transparent",
          color: "#aaa",
          cursor: "pointer",
          fontSize: 12,
        }}
      >
        ↩ 撤销
      </button>
      <button
        onClick={() => quickAction("redo")}
        disabled={status !== "idle"}
        style={{
          padding: "6px 12px",
          borderRadius: 6,
          border: "1px solid #0f3460",
          background: "transparent",
          color: "#aaa",
          cursor: "pointer",
          fontSize: 12,
        }}
      >
        ↪ 重做
      </button>
    </footer>
  );
}
