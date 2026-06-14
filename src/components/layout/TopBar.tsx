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
