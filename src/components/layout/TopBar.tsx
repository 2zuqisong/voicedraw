import { useAppStore } from "../../store";

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

  return (
    <div
      style={{
        position: "absolute",
        top: 16,
        left: 20,
        zIndex: 80,
        display: "flex",
        alignItems: "center",
        gap: 12,
        fontFamily: "var(--font-mono)",
        fontSize: 20,
        fontWeight: 300,
        letterSpacing: "0.02em",
        color: "var(--text-secondary)",
        userSelect: "none",
        pointerEvents: "none",
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
  );
}
