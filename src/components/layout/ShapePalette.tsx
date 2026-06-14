import { useState } from "react";
import { useAppStore } from "../../store";

const TYPE_NAME: Record<string, string> = {
  circle: "圆形", rectangle: "矩形", triangle: "三角形", line: "线段", dot: "点",
  house: "房子", sun: "太阳", tree: "树", smiley: "笑脸", star: "星星",
  cake: "蛋糕", gift: "礼物盒", balloon: "气球", candle: "蜡烛",
  heart: "爱心", flower: "花朵", arrow_shape: "箭头",
  speech_bubble: "对话气泡", cloud: "云朵", lightning: "闪电",
  Start: "开始", End: "结束", Process: "处理", Decision: "判断",
  Data: "数据", Subprocess: "子流程", Text: "文字",
};

function getTypeName(shapeType?: string, nodeType?: string): string {
  if (shapeType) return TYPE_NAME[shapeType] ?? shapeType;
  if (nodeType) return TYPE_NAME[nodeType] ?? nodeType;
  return "未知";
}

export default function ShapePalette() {
  const [collapsed, setCollapsed] = useState(false);
  const canvasState = useAppStore((s) => s.canvasState);
  const nodes = canvasState?.nodes ?? {};
  const nodeList = Object.values(nodes);

  return (
    <div style={{
      position: "absolute",
      left: 12,
      top: "50%",
      transform: "translateY(-50%)",
      maxHeight: "calc(100vh - 200px)",
      zIndex: 80,
      display: "flex",
      flexDirection: "column",
      background: "var(--surface, #fff)",
      border: "1px solid var(--border, #e2e2de)",
      borderRadius: "var(--radius, 8px)",
      boxShadow: "0 2px 16px rgba(0,0,0,0.06)",
      overflow: "hidden",
      transition: "width 0.2s ease",
      width: collapsed ? 36 : 160,
      userSelect: "none",
    }}>
      {/* 头部 */}
      <button
        onClick={() => setCollapsed(!collapsed)}
        title={collapsed ? "展开" : "收起"}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 6,
          padding: "10px 12px",
          border: "none",
          borderBottom: collapsed ? "none" : "1px solid var(--border-light, #efefeb)",
          background: "transparent",
          cursor: "pointer",
          fontSize: 12,
          fontWeight: 600,
          color: "var(--text-secondary, #6b6b6b)",
          whiteSpace: "nowrap",
          justifyContent: collapsed ? "center" : "flex-start",
        }}
      >
        <span style={{ flexShrink: 0 }}>{collapsed ? "▸" : "◂"}</span>
        {!collapsed && `画布内容 (${nodeList.length})`}
      </button>

      {/* 列表 */}
      {!collapsed && (
        <div style={{ flex: 1, overflowY: "auto", padding: "4px 0" }}>
          {nodeList.length === 0 ? (
            <div style={{
              padding: "16px 12px",
              fontSize: 11,
              color: "var(--text-tertiary, #b0b0b0)",
              textAlign: "center",
              lineHeight: 1.6,
            }}>
              画布为空
            </div>
          ) : (
            nodeList.map((node) => {
              const typeName = getTypeName(node.shape_type, node.node_type);
              const displayLabel = node.label || typeName;
              return (
                <div
                  key={node.id}
                  title={node.label ? `${typeName} · "${node.label}"` : typeName}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: 8,
                    padding: "5px 12px",
                    cursor: "default",
                    fontSize: 12,
                    color: "var(--text-primary, #141414)",
                    lineHeight: 1.4,
                  }}
                >
                  <span style={{
                    flexShrink: 0,
                    width: 6, height: 6,
                    borderRadius: 3,
                    background: node.shape_type ? "var(--accent, #e8960a)" : "var(--text-tertiary, #b0b0b0)",
                  }} />
                  <span style={{
                    fontWeight: 500,
                    whiteSpace: "nowrap",
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                  }}>
                    {displayLabel}
                  </span>
                </div>
              );
            })
          )}
        </div>
      )}
    </div>
  );
}
