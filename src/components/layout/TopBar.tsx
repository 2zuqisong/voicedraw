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

        {/* settings gear */}
        <button
          onClick={toggleSettings}
          title="API 设置"
          style={{
            background: "none",
            border: "none",
            cursor: "pointer",
            fontSize: 15,
            padding: "2px 4px",
            lineHeight: 1,
            color: "var(--text-secondary)",
            opacity: 0.45,
            transition: "opacity 0.15s",
          }}
          onMouseEnter={(e) => (e.currentTarget.style.opacity = "1")}
          onMouseLeave={(e) => (e.currentTarget.style.opacity = "0.45")}
        >
          ⚙
        </button>

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

      <SettingsPanel open={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </>
  );
}
