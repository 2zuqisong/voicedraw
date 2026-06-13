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
      const timer = setTimeout(() => setVisible(false), 3000);
      return () => clearTimeout(timer);
    }
  }, [lastOperation, status]);

  if (!visible) return null;

  const bgColor =
    status === "error"
      ? "#dc3545"
      : status === "executing"
        ? "#28a745"
        : status === "thinking"
          ? "#17a2b8"
          : "#666";

  const icon =
    status === "error" ? "❌ " : status === "executing" ? "✅ " : "💬 ";

  return (
    <div
      style={{
        position: "fixed",
        top: 48,
        left: "50%",
        transform: "translateX(-50%)",
        background: bgColor,
        color: "#fff",
        padding: "8px 20px",
        borderRadius: 20,
        fontSize: 13,
        zIndex: 1000,
        boxShadow: "0 2px 8px rgba(0,0,0,0.3)",
        animation: "fadeInDown 0.3s ease",
      }}
    >
      {icon}
      {message}
      <style>{`
        @keyframes fadeInDown {
          from { opacity: 0; transform: translateX(-50%) translateY(-10px); }
          to { opacity: 1; transform: translateX(-50%) translateY(0); }
        }
      `}</style>
    </div>
  );
}
