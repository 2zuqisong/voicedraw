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
      background: "#16213e",
      borderBottom: "1px solid #0f3460",
      fontSize: 13,
      flexShrink: 0,
    }}>
      <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
        <span style={{ fontWeight: 700, fontSize: 15 }}>🎤 VoiceDraw</span>
        {canvasState && (
          <span style={{ color: "#aaa" }}>[{canvasState.title}]</span>
        )}
      </div>
      <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
        <span style={{ color: "#888", fontSize: 12, maxWidth: 300, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {lastOp}
        </span>
        <StatusLight status={status} />
      </div>
    </div>
  );
}