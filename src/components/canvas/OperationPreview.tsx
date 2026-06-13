import { useState } from "react";
import { useAppStore } from "../../store";

export default function OperationPreview() {
  const pendingPlan = useAppStore((s) => s.pendingPlan);
  const status = useAppStore((s) => s.status);
  const confirmPlan = useAppStore((s) => s.confirmPlan);
  const cancelPlan = useAppStore((s) => s.cancelPlan);
  const modifyPlan = useAppStore((s) => s.modifyPlan);
  const [editText, setEditText] = useState("");
  const [isEditing, setIsEditing] = useState(false);

  const isExecuting = status === "executing";

  if (!pendingPlan) return null;

  const gridPos = pendingPlan.grid_position
    ? `grid(${pendingPlan.grid_position[0]},${pendingPlan.grid_position[1]})`
    : "auto";

  const edgePreviews = pendingPlan.edges
    .slice(0, 4)
    .map((e) => `${e.from}→${e.to}${e.label ? `:${e.label}` : ""}`)
    .join(", ");

  return (
    <div
      style={{
        position: "absolute",
        top: 48,
        left: "50%",
        transform: "translateX(-50%)",
        zIndex: 200,
        background: "var(--surface)",
        border: "1px solid var(--border)",
        borderRadius: "var(--radius-panel)",
        padding: "16px 20px 14px",
        minWidth: 380,
        maxWidth: 500,
        fontFamily: "var(--font-mono)",
        fontSize: 11,
        fontWeight: 300,
        color: "var(--text-secondary)",
        letterSpacing: "0.01em",
        lineHeight: 1.7,
      }}
    >
      {/* Header */}
      <div
        style={{
          fontWeight: 500,
          color: "var(--text-primary)",
          marginBottom: 10,
          fontSize: 12,
          letterSpacing: "0.03em",
        }}
      >
        pending plan
      </div>

      {/* Details */}
      <div style={{ marginBottom: 12 }}>
        <div>
          <span style={{ color: "var(--text-tertiary)" }}>type </span>
          {pendingPlan.diagram_type || "flowchart"}
        </div>
        <div>
          <span style={{ color: "var(--text-tertiary)" }}>nodes </span>
          {pendingPlan.nodes.map((n) => n.label).join(", ")}
          <span style={{ color: "var(--text-tertiary)" }}> ({pendingPlan.nodes.length})</span>
        </div>
        {pendingPlan.edges.length > 0 && (
          <div>
            <span style={{ color: "var(--text-tertiary)" }}>edges </span>
            {edgePreviews}
            {pendingPlan.edges.length > 4 && ` +${pendingPlan.edges.length - 4}`}
          </div>
        )}
        <div>
          <span style={{ color: "var(--text-tertiary)" }}>pos </span>
          {gridPos}
        </div>
        <div>
          <span style={{ color: "var(--text-tertiary)" }}>layout </span>
          {pendingPlan.layout_direction === "left_right" ? "left→right" : "top→down"}
        </div>
      </div>

      {/* Actions */}
      {isEditing ? (
        <div style={{ display: "flex", gap: 6 }}>
          <input
            type="text"
            value={editText}
            onChange={(e) => setEditText(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") {
                modifyPlan(editText);
                setEditText("");
                setIsEditing(false);
              }
              if (e.key === "Escape") {
                setIsEditing(false);
                setEditText("");
              }
            }}
            placeholder="modify then enter…"
            autoFocus
            style={{
              flex: 1,
              height: 28,
              border: "1px solid var(--accent)",
              borderRadius: "var(--radius)",
              padding: "0 8px",
              fontFamily: "var(--font-mono)",
              fontSize: 11,
              fontWeight: 300,
              outline: "none",
              background: "var(--surface)",
              color: "var(--text-primary)",
            }}
          />
          <button
            onClick={() => {
              modifyPlan(editText);
              setEditText("");
              setIsEditing(false);
            }}
            className="btn-accent"
            style={{ padding: "0 12px", height: 28 }}
          >
            update
          </button>
        </div>
      ) : (
        <div style={{ display: "flex", gap: 6 }}>
          <button
            onClick={confirmPlan}
            disabled={isExecuting}
            className="btn-accent"
            style={{ flex: 1, height: 28 }}
          >
            {isExecuting ? "running…" : "confirm"}
          </button>
          <button
            onClick={cancelPlan}
            disabled={isExecuting}
            className="btn-ghost"
            style={{ padding: "0 14px", height: 28 }}
          >
            cancel
          </button>
          <button
            onClick={() => {
              setEditText("");
              setIsEditing(true);
            }}
            disabled={isExecuting}
            className="btn-ghost"
            style={{ padding: "0 8px", height: 28, width: 28 }}
          >
            ✎
          </button>
        </div>
      )}
    </div>
  );
}
