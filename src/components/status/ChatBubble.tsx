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
      // 30 秒后自动淡出
      const timer = setTimeout(() => setFadeOut(true), 30000);
      return () => clearTimeout(timer);
    }
  }, [conversation]);

  // 用户开始新指令时隐藏
  useEffect(() => {
    if (status === "thinking" || status === "listening") {
      setFadeOut(true);
    }
  }, [status]);

  if (!visible || !message) return null;

  return (
    <div
      onClick={() => {
        // 点击气泡关闭
        setFadeOut(true);
        setTimeout(() => setVisible(false), 400);
      }}
      style={{
        position: "absolute",
        bottom: 80,
        left: "50%",
        transform: "translateX(-50%)",
        maxWidth: 500,
        minWidth: 200,
        padding: "10px 16px",
        borderRadius: 18,
        background: "rgba(255,255,255,0.95)",
        boxShadow: "0 2px 12px rgba(0,0,0,0.08)",
        border: "1px solid #e8eaed",
        fontSize: 13,
        lineHeight: 1.6,
        color: "#202124",
        cursor: "pointer",
        zIndex: 110,
        userSelect: "text",
        opacity: fadeOut ? 0 : 1,
        transition: "opacity 0.4s",
        backdropFilter: "blur(4px)",
        wordBreak: "break-word",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "flex-start",
          gap: 8,
        }}
      >
        <span style={{ fontSize: 18, flexShrink: 0 }}>🤖</span>
        <span>{message}</span>
      </div>
      <div
        style={{
          fontSize: 10,
          color: "#9aa0a6",
          textAlign: "right",
          marginTop: 4,
        }}
      >
        点击关闭
      </div>
    </div>
  );
}
