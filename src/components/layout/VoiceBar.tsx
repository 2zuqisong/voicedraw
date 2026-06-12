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
