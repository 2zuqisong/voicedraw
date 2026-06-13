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
        background: "rgba(0,0,0,0.6)",
        color: "#fff",
        padding: "16px 32px",
        borderRadius: 12,
        fontSize: 15,
        zIndex: 100,
      }}
    >
      🤔 正在理解指令...
    </div>
  );
}
