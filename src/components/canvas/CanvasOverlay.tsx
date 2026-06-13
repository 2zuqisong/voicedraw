import { useAppStore } from "../../store";

export default function CanvasOverlay() {
  const status = useAppStore((s) => s.status);

  if (status !== "thinking") return null;

  return (
    <div
      style={{
        position: "absolute",
        top: "50%",
        left: "50%",
        transform: "translate(-50%, -50%)",
        zIndex: 100,
        fontFamily: "var(--font-mono)",
        fontSize: 18,
        fontWeight: 300,
        color: "var(--text-tertiary)",
        letterSpacing: "0.04em",
        animation: "pulse 1.8s ease-in-out infinite",
        userSelect: "none",
        pointerEvents: "none",
      }}
    >
      thinking…
    </div>
  );
}
