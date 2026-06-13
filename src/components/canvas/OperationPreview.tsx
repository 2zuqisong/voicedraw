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
    ? `(${pendingPlan.grid_position[0]}, ${pendingPlan.grid_position[1]})`
    : "自动";

  return (
    <div
      style={{
        position: "absolute",
        top: 60,
        left: "50%",
        transform: "translateX(-50%)",
        zIndex: 200,
        background: "rgba(255,255,255,0.96)",
        borderRadius: 16,
        boxShadow:
          "0 4px 24px rgba(0,0,0,0.12), 0 1px 8px rgba(0,0,0,0.08)",
        padding: "16px 20px",
        minWidth: 380,
        maxWidth: 480,
        backdropFilter: "blur(8px)",
      }}
    >
      {/* 标题 */}
      <div
        style={{
          display: "flex",
          alignItems: "center",
          gap: 8,
          marginBottom: 12,
        }}
      >
        <span style={{ fontSize: 18 }}>📋</span>
        <span
          style={{
            fontSize: 14,
            fontWeight: 600,
            color: "#202124",
          }}
        >
          即将执行的操作
        </span>
      </div>

      {/* 操作摘要 */}
      <div
        style={{
          fontSize: 13,
          color: "#5f6368",
          lineHeight: 1.6,
          marginBottom: 8,
        }}
      >
        <div>
          <span style={{ color: "#9aa0a6" }}>类型：</span>
          {pendingPlan.diagram_type}
        </div>
        <div>
          <span style={{ color: "#9aa0a6" }}>节点：</span>
          {pendingPlan.nodes.map((n) => n.label).join("、")}
          <span style={{ color: "#9aa0a6" }}>
            （共 {pendingPlan.nodes.length} 个）
          </span>
        </div>
        {pendingPlan.edges.length > 0 && (
          <div>
            <span style={{ color: "#9aa0a6" }}>连线：</span>
            {pendingPlan.edges
              .slice(0, 3)
              .map(
                (e) =>
                  `${e.from}→${e.to}${e.label ? `(${e.label})` : ""}`
              )
              .join(", ")}
            {pendingPlan.edges.length > 3 &&
              ` ... 等${pendingPlan.edges.length}条`}
          </div>
        )}
        <div>
          <span style={{ color: "#9aa0a6" }}>位置：</span>
          网格 {gridPos}
        </div>
        <div>
          <span style={{ color: "#9aa0a6" }}>布局：</span>
          {pendingPlan.layout_direction === "top_down" ? "上→下" : "左→右"}
        </div>
      </div>

      {/* 操作按钮 */}
      {isEditing ? (
        <div style={{ display: "flex", gap: 8 }}>
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
            placeholder="修改指令后按回车..."
            autoFocus
            style={{
              flex: 1,
              height: 36,
              borderRadius: 20,
              border: "1px solid #667eea",
              padding: "0 14px",
              fontSize: 13,
              outline: "none",
            }}
          />
          <button
            onClick={() => {
              modifyPlan(editText);
              setEditText("");
              setIsEditing(false);
            }}
            style={{
              padding: "0 16px",
              height: 36,
              borderRadius: 20,
              border: "none",
              background: "#667eea",
              color: "#fff",
              fontSize: 13,
              cursor: "pointer",
            }}
          >
            修改
          </button>
        </div>
      ) : (
        <div
          style={{
            display: "flex",
            gap: 10,
            justifyContent: "center",
          }}
        >
          <button
            onClick={confirmPlan}
            disabled={isExecuting}
            style={{
              flex: 1,
              height: 36,
              borderRadius: 20,
              border: "none",
              background: isExecuting ? "#a0d4aa" : "#34a853",
              color: "#fff",
              fontSize: 14,
              fontWeight: 500,
              cursor: isExecuting ? "default" : "pointer",
            }}
          >
            {isExecuting ? "⏳ 执行中..." : "✓ 确认执行"}
          </button>
          <button
            onClick={cancelPlan}
            disabled={isExecuting}
            style={{
              width: 72,
              height: 36,
              borderRadius: 20,
              border: "1px solid #e8eaed",
              background: "#fff",
              color: isExecuting ? "#ccc" : "#ea4335",
              fontSize: 13,
              cursor: isExecuting ? "default" : "pointer",
            }}
          >
            ✕ 取消
          </button>
          <button
            onClick={() => {
              setEditText("");
              setIsEditing(true);
            }}
            disabled={isExecuting}
            style={{
              width: 72,
              height: 36,
              borderRadius: 20,
              border: "1px solid #e8eaed",
              background: "#fff",
              color: isExecuting ? "#ccc" : "#5f6368",
              fontSize: 13,
              cursor: isExecuting ? "default" : "pointer",
            }}
          >
            ✎ 修改
          </button>
        </div>
      )}
    </div>
  );
}
