import { useState, useCallback } from "react";
import { useAppStore } from "../../store";
import SettingsPanel from "./SettingsPanel";

const STATUS_LABEL: Record<string, string> = {
  idle: "",
  listening: "● rec",
  thinking: "● thinking",
  executing: "● draw",
  error: "● err",
};

export default function TopBar() {
  const status = useAppStore((s) => s.status);
  const canvasMode = useAppStore((s) => s.canvasMode);
  const setCanvasMode = useAppStore((s) => s.setCanvasMode);
  const isActive = status !== "idle";
  const [settingsOpen, setSettingsOpen] = useState(false);
  const toggleSettings = useCallback(() => setSettingsOpen((v) => !v), []);

  return (
    <>
      {/* title — top-left */}
      <div
        style={{
          position: "absolute",
          top: 16,
          left: 20,
          zIndex: 80,
          display: "flex",
          alignItems: "center",
          gap: 10,
          fontFamily: "var(--font-mono)",
          fontSize: 20,
          fontWeight: 300,
          letterSpacing: "0.02em",
          color: "var(--text-secondary)",
          userSelect: "none",
        }}
      >
        <span style={{
          fontWeight: 500,
          color: "var(--text-primary)",
          letterSpacing: "-0.01em",
        }}>
          voice-draw
        </span>

        {isActive && (
          <span style={{
            color: "var(--accent)",
            animation: "pulse 1.5s ease-in-out infinite",
            fontSize: 14,
          }}>
            {STATUS_LABEL[status] || ""}
          </span>
        )}
      </div>

      {/* mode toggle — top-center */}
      <div
        style={{
          position: "absolute",
          top: 16,
          left: "50%",
          transform: "translateX(-50%)",
          zIndex: 200,
          display: "flex",
          background: "var(--surface, #fff)",
          border: "1px solid var(--border, #d4d4ce)",
          borderRadius: "var(--radius, 6px)",
          boxShadow: "0 1px 8px rgba(0,0,0,0.08)",
          overflow: "hidden",
          userSelect: "none",
        }}
      >
        <button
          onClick={() => setCanvasMode("vector")}
          style={{
            padding: "6px 16px",
            fontSize: 13,
            fontWeight: canvasMode === "vector" ? 600 : 400,
            border: "none",
            background: canvasMode === "vector" ? "var(--accent, #e8960a)" : "transparent",
            color: canvasMode === "vector" ? "#fff" : "var(--text-secondary, #6b6b6b)",
            cursor: "pointer",
            fontFamily: "var(--font-mono)",
          }}
        >
          矢量
        </button>
        <button
          onClick={() => setCanvasMode("pixel")}
          style={{
            padding: "6px 16px",
            fontSize: 13,
            fontWeight: canvasMode === "pixel" ? 600 : 400,
            border: "none",
            background: canvasMode === "pixel" ? "var(--accent, #e8960a)" : "transparent",
            color: canvasMode === "pixel" ? "#fff" : "var(--text-secondary, #6b6b6b)",
            cursor: "pointer",
            fontFamily: "var(--font-mono)",
          }}
        >
          像素
        </button>
      </div>

      {/* settings — top-right */}
      <button
        onClick={toggleSettings}
        title="API 设置"
        style={{
          position: "absolute",
          top: 12,
          right: 16,
          zIndex: 80,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          width: 40,
          height: 40,
          fontSize: 22,
          lineHeight: 1,
          background: "var(--surface, #fff)",
          border: "1px solid var(--border, #d4d4ce)",
          borderRadius: "var(--radius, 4px)",
          color: "var(--text-primary, #1a1a1a)",
          cursor: "pointer",
          opacity: 0.8,
          transition: "opacity 0.15s, background 0.15s",
        }}
        onMouseEnter={(e) => {
          e.currentTarget.style.opacity = "1";
          e.currentTarget.style.background = "var(--border, #e2e2de)";
        }}
        onMouseLeave={(e) => {
          e.currentTarget.style.opacity = "0.8";
          e.currentTarget.style.background = "var(--surface, #fff)";
        }}
      >
        ⚙
      </button>

      <SettingsPanel open={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </>
  );
}
