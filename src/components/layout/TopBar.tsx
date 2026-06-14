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
      <div style={{
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        height: 44,
        padding: "0 16px",
        flexShrink: 0,
        userSelect: "none",
      }}>
        {/* left — title + status */}
        <div style={{
          display: "flex",
          alignItems: "center",
          gap: 10,
          fontFamily: "var(--font-mono)",
          fontSize: 16,
          fontWeight: 300,
          letterSpacing: "0.02em",
          color: "var(--text-secondary)",
          width: 180,
        }}>
          <span style={{ fontWeight: 500, color: "var(--text-primary)" }}>
            voice-draw
          </span>
          {isActive && (
            <span style={{
              color: "var(--accent)",
              animation: "pulse 1.5s ease-in-out infinite",
              fontSize: 12,
            }}>
              {STATUS_LABEL[status] || ""}
            </span>
          )}
        </div>

        {/* center — mode toggle */}
        <div style={{
          display: "flex",
          background: "var(--surface, #fff)",
          border: "1px solid var(--border, #d4d4ce)",
          borderRadius: 6,
          overflow: "hidden",
          boxShadow: "0 1px 4px rgba(0,0,0,0.06)",
        }}>
          <button
            onClick={() => setCanvasMode("vector")}
            style={{
              padding: "5px 14px",
              fontSize: 13,
              fontWeight: canvasMode === "vector" ? 600 : 400,
              border: "none",
              background: canvasMode === "vector" ? "var(--accent, #e8960a)" : "transparent",
              color: canvasMode === "vector" ? "#fff" : "var(--text-secondary, #6b6b6b)",
              cursor: "pointer",
              fontFamily: "var(--font-mono)",
              transition: "background 0.1s",
            }}
          >
            矢量
          </button>
          <button
            onClick={() => setCanvasMode("pixel")}
            style={{
              padding: "5px 14px",
              fontSize: 13,
              fontWeight: canvasMode === "pixel" ? 600 : 400,
              border: "none",
              background: canvasMode === "pixel" ? "var(--accent, #e8960a)" : "transparent",
              color: canvasMode === "pixel" ? "#fff" : "var(--text-secondary, #6b6b6b)",
              cursor: "pointer",
              fontFamily: "var(--font-mono)",
              transition: "background 0.1s",
            }}
          >
            像素
          </button>
        </div>

        {/* right — settings */}
        <div style={{ width: 180, display: "flex", justifyContent: "flex-end" }}>
          <button
            onClick={toggleSettings}
            title="API 设置"
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              width: 36,
              height: 36,
              fontSize: 18,
              background: "transparent",
              border: "1px solid var(--border, #d4d4ce)",
              borderRadius: 4,
              color: "var(--text-secondary)",
              cursor: "pointer",
              opacity: 0.7,
            }}
            onMouseEnter={(e) => { e.currentTarget.style.opacity = "1"; }}
            onMouseLeave={(e) => { e.currentTarget.style.opacity = "0.7"; }}
          >
            ⚙
          </button>
        </div>
      </div>

      <SettingsPanel open={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </>
  );
}
