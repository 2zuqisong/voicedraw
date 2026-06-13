import { useEffect, useState } from "react";
import { useAppStore } from "../../store";

export default function ChatBubble() {
  const conversation = useAppStore((s) => s.conversation);
  const status = useAppStore((s) => s.status);
  const [visible, setVisible] = useState(false);
  const [message, setMessage] = useState("");
  const [fadeOut, setFadeOut] = useState(false);

  useEffect(() => {
    if (conversation.length === 0) return;
    const last = conversation[conversation.length - 1];
    if (last.role === "assistant" && last.content) {
      setMessage(last.content);
      setVisible(true);
      setFadeOut(false);
      const timer = setTimeout(() => setFadeOut(true), 30000);
      return () => clearTimeout(timer);
    }
  }, [conversation]);

  useEffect(() => {
    if (status === "thinking" || status === "listening") {
      setFadeOut(true);
    }
  }, [status]);

  if (!visible || !message) return null;

  return (
    <div
      onClick={() => {
        setFadeOut(true);
        setTimeout(() => setVisible(false), 300);
      }}
      style={{
        position: "absolute",
        bottom: 100,
        left: "50%",
        transform: "translateX(-50%)",
        maxWidth: 480,
        minWidth: 180,
        padding: "8px 14px",
        background: "var(--surface)",
        border: "1px solid var(--border)",
        borderRadius: "var(--radius-panel)",
        fontFamily: "var(--font-mono)",
        fontSize: 11,
        fontWeight: 300,
        lineHeight: 1.7,
        color: "var(--text-secondary)",
        cursor: "pointer",
        zIndex: 110,
        userSelect: "text",
        opacity: fadeOut ? 0 : 1,
        transition: "opacity 0.25s",
        wordBreak: "break-word",
        letterSpacing: "0.01em",
      }}
    >
      {message}
    </div>
  );
}
