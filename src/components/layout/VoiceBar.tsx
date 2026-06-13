import { useCallback, useRef, useState } from "react";
import { useAppStore } from "../../store";
import { useVoiceRecognition } from "../../hooks/useVoiceRecognition";
import WaveformVisualizer from "../voice/WaveformVisualizer";

export default function VoiceBar() {
  const isListening = useAppStore((s) => s.isListening);
  const transcript = useAppStore((s) => s.transcript);
  const status = useAppStore((s) => s.status);
  const setTranscript = useAppStore((s) => s.setTranscript);
  const setStatus = useAppStore((s) => s.setStatus);
  const submitCommand = useAppStore((s) => s.submitCommand);
  const quickAction = useAppStore((s) => s.quickAction);

  const lastTextRef = useRef("");
  const [draftText, setDraftText] = useState("");

  const handleResult = useCallback((text: string, _isFinal: boolean) => {
    setTranscript(text);
    lastTextRef.current = text;
  }, [setTranscript]);

  const handleError = useCallback((error: string) => {
    console.error("STT Error:", error);
    setStatus("error");
    setTimeout(() => setStatus("idle"), 2000);
  }, [setStatus]);

  const handleEnd = useCallback(() => {}, []);

  const { start, stop, abort, isSupported } = useVoiceRecognition({
    onResult: handleResult,
    onError: handleError,
    onEnd: handleEnd,
  });

  const handleStart = () => {
    if (status !== "idle") return;
    setTranscript("");
    useAppStore.setState({ isListening: true, status: "listening" });
    start();
  };

  const handleConfirm = () => {
    stop();
    useAppStore.setState({ isListening: false });
    const text = lastTextRef.current.trim();
    if (text.length > 0) submitCommand(text);
  };

  const handleCancel = () => {
    abort();
    setTranscript("");
    useAppStore.setState({ isListening: false, status: "idle" });
  };

  const handleTextSubmit = () => {
    const text = draftText.trim();
    if (text.length === 0 || status !== "idle") return;
    setDraftText("");
    submitCommand(text);
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter") handleTextSubmit();
  };

  const isBusy = status !== "idle" && status !== "listening";

  // Shared button dims
  const btnSize = 40;

  return (
    <div
      style={{
        position: "absolute",
        bottom: 32,
        left: "50%",
        transform: "translateX(-50%)",
        display: "flex",
        alignItems: "center",
        gap: 12,
        zIndex: 100,
        userSelect: "none",
      }}
    >
      {/* Undo / Redo */}
      <div style={{ display: "flex", gap: 6 }}>
        <button
          onClick={() => quickAction("undo")}
          disabled={status !== "idle"}
          className="btn-ghost"
          style={{ width: btnSize, height: btnSize, padding: 0, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 18 }}
          title="撤销"
        >
          ↺
        </button>
        <button
          onClick={() => quickAction("redo")}
          disabled={status !== "idle"}
          className="btn-ghost"
          style={{ width: btnSize, height: btnSize, padding: 0, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 18 }}
          title="重做"
        >
          ↻
        </button>
      </div>

      {/* Main pill */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 10,
          background: "var(--surface)",
          border: isListening ? "1px solid var(--accent)" : "1px solid var(--border)",
          borderRadius: "var(--radius)",
          padding: "8px 12px 8px 20px",
          minWidth: 560,
          transition: "border-color 0.15s",
        }}
      >
        <WaveformVisualizer isActive={isListening} barCount={20} />

        {isSupported ? (
          <>
            {isListening && (
              <button
                onClick={handleCancel}
                className="btn-ghost"
                style={{ width: btnSize, height: btnSize, padding: 0, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 20, flexShrink: 0, fontWeight: 300 }}
                title="取消"
              >
                ✕
              </button>
            )}

            {/* Transcript / prompt */}
            <div
              onClick={isListening ? undefined : handleStart}
              style={{
                flex: 1,
                height: btnSize,
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                cursor: isBusy ? "default" : "pointer",
                padding: "0 12px",
                overflow: "hidden",
                background: isListening ? "var(--accent-dim)" : "transparent",
                borderRadius: "var(--radius)",
                transition: "background 0.15s",
              }}
            >
              <span
                style={{
                  fontFamily: "var(--font-mono)",
                  fontSize: 16,
                  fontWeight: 300,
                  color: isListening ? "var(--accent)" : "var(--text-tertiary)",
                  whiteSpace: "nowrap",
                  textOverflow: "ellipsis",
                  overflow: "hidden",
                  letterSpacing: "0.02em",
                }}
              >
                {isListening
                  ? (transcript || "…")
                  : status === "idle"
                    ? "click to speak"
                    : status === "thinking"
                      ? "thinking…"
                      : status === "executing"
                        ? "drawing…"
                        : "error"}
              </span>
            </div>

            {isListening && (
              <button
                onClick={handleConfirm}
                disabled={transcript.trim().length === 0}
                className={transcript.trim().length > 0 ? "btn-accent" : "btn-ghost"}
                style={{ width: btnSize, height: btnSize, padding: 0, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 20, flexShrink: 0 }}
                title="确认"
              >
                ✓
              </button>
            )}
          </>
        ) : (
          <>
            <span style={{ fontFamily: "var(--font-mono)", fontSize: 14, color: "var(--text-tertiary)", flexShrink: 0 }}>
              ⌨
            </span>
            <input
              type="text"
              value={draftText}
              onChange={(e) => setDraftText(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder="type then enter…"
              disabled={status !== "idle"}
              style={{
                flex: 1,
                height: btnSize,
                border: "none",
                background: "transparent",
                padding: "0 10px",
                fontFamily: "var(--font-mono)",
                fontSize: 16,
                fontWeight: 300,
                outline: "none",
                color: "var(--text-primary)",
                borderRadius: 0,
              }}
            />
            <button
              onClick={handleTextSubmit}
              disabled={draftText.trim().length === 0 || status !== "idle"}
              className={draftText.trim().length > 0 && status === "idle" ? "btn-accent" : "btn-ghost"}
              style={{ width: btnSize, height: btnSize, padding: 0, display: "flex", alignItems: "center", justifyContent: "center", fontSize: 20, flexShrink: 0 }}
              title="发送"
            >
              →
            </button>
          </>
        )}
      </div>
    </div>
  );
}
