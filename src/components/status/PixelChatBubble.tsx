import { useAppStore } from "../../store";

/**
 * 像素模式专用回应气泡 — 在画布顶部显示 LLM 的回复文字。
 */
export default function PixelChatBubble() {
  const lastOperation = useAppStore((s) => s.lastOperation);
  const status = useAppStore((s) => s.status);

  if (!lastOperation && status === "idle") return null;

  const isBusy = status === "thinking" || status === "executing";

  return (
    <div
      style={{
        position: "absolute",
        top: 72,
        left: "50%",
        transform: "translateX(-50%)",
        zIndex: 90,
        maxWidth: 320,
        padding: "6px 16px",
        borderRadius: 16,
        background: isBusy ? "rgba(0,0,0,0.06)" : "rgba(0,0,0,0.03)",
        fontSize: 12,
        color: "var(--text-secondary, #6b6b6b)",
        textAlign: "center",
        fontFamily: "var(--font-mono)",
        pointerEvents: "none",
        userSelect: "none",
        whiteSpace: "nowrap",
        overflow: "hidden",
        textOverflow: "ellipsis",
        transition: "opacity 0.3s",
        opacity: isBusy ? 0.7 : 0.5,
      }}
    >
      {isBusy ? (status === "thinking" ? "thinking…" : "drawing…") : lastOperation}
    </div>
  );
}
