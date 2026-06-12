import { useAppStore } from "../../store";
import StatusLight from "../status/StatusLight";

export default function TopBar() {
  const canvasState = useAppStore((s) => s.canvasState);
  const status = useAppStore((s) => s.status);
  const lastOp = useAppStore((s) => s.lastOperation);

  return (
    <div style={{
      height: 36,
      display: "flex",
      alignItems: "center",
      justifyContent: "space-between",
      padding: "0 16px",
      background: "#ffffff",
      borderBottom: "1px solid #e8eaed",
      fontSize: 13,
      flexShrink: 0,
    }}>
      <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
        <span style={{ fontWeight: 700, fontSize: 15, color: "#1f2328" }}>🎤 VoiceDraw</span>
        {canvasState && (
          <span style={{ color: "#9aa0a6" }}>[{canvasState.title}]</span>
        )}
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
        <span style={{ color: "#9aa0a6", fontSize: 12, maxWidth: 300, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {lastOp}
        </span>
        <StatusLight status={status} />
      </div>
    </div>
  );
}