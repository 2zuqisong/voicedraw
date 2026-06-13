import { useEffect, useState } from "react";
import { useAppStore } from "../../store";

export default function Toast() {
  const lastOperation = useAppStore((s) => s.lastOperation);
  const status = useAppStore((s) => s.status);
  const [visible, setVisible] = useState(false);
  const [message, setMessage] = useState("");

  useEffect(() => {
    if (lastOperation && status !== "idle") {
      setMessage(lastOperation);
      setVisible(true);
      const timer = setTimeout(() => setVisible(false), 2800);
      return () => clearTimeout(timer);
    }
  }, [lastOperation, status]);

  if (!visible) return null;

  const prefix =
    status === "error" ? "err:" : status === "executing" ? "ok:" : "";

  return (
    <div
      style={{
        position: "fixed",
        top: 16,
        left: "50%",
        transform: "translateX(-50%)",
        zIndex: 1000,
        fontFamily: "var(--font-mono)",
        fontSize: 11,
        fontWeight: 300,
        letterSpacing: "0.02em",
        color: status === "error" ? "var(--text-primary)" : "var(--text-secondary)",
        background: "var(--surface)",
        border: `1px solid ${status === "error" ? "var(--text-primary)" : "var(--border)"}`,
        borderRadius: "var(--radius)",
        padding: "4px 12px",
        animation: "fadeIn 0.2s ease",
        maxWidth: 480,
        overflow: "hidden",
        textOverflow: "ellipsis",
        whiteSpace: "nowrap",
      }}
    >
      {prefix}
      {message}
    </div>
  );
}
