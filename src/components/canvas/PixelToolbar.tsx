import { useAppStore } from "../../store";
import type { PixelTool } from "../../store/types";

const PRESET_COLORS = [
  "#1a1a1a", "#ffffff", "#e03131", "#f08c00", "#fcc419",
  "#40c057", "#228be6", "#7950f2", "#e64980", "#82c91e",
  "#15aabf", "#fab005", "#fd7e14", "#be4bdb", "#868e96",
];

const TOOLS: { key: PixelTool; label: string; icon: string }[] = [
  { key: "pencil", label: "铅笔", icon: "✎" },
  { key: "eraser", label: "橡皮", icon: "⌫" },
  { key: "fill", label: "填充", icon: "🪣" },
  { key: "picker", label: "取色", icon: "💉" },
];

const CELL_SIZES = [10, 16, 20, 32, 40];

export default function PixelToolbar() {
  const pixel = useAppStore((s) => s.pixel);
  const setPixelTool = useAppStore((s) => s.setPixelTool);
  const setPixelColor = useAppStore((s) => s.setPixelColor);
  const setPixelCellSize = useAppStore((s) => s.setPixelCellSize);
  const clearPixelData = useAppStore((s) => s.clearPixelData);
  const pixelUndo = useAppStore((s) => s.pixelUndo);

  const { tool, color, cellSize, cols, rows, undoStack } = pixel;

  return (
    <div
      style={{
        position: "absolute",
        bottom: 28,
        left: "50%",
        transform: "translateX(-50%)",
        display: "flex",
        alignItems: "center",
        gap: 10,
        padding: "8px 16px",
        background: "var(--surface, #fff)",
        border: "1px solid var(--border, #e2e2de)",
        borderRadius: "var(--radius, 8px)",
        boxShadow: "0 2px 12px rgba(0,0,0,0.06)",
        zIndex: 100,
        userSelect: "none",
      }}
    >
      {/* 工具按钮 */}
      <div style={{ display: "flex", gap: 2 }}>
        {TOOLS.map((t) => (
          <button
            key={t.key}
            onClick={() => setPixelTool(t.key)}
            title={t.label}
            style={{
              width: 36,
              height: 36,
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: t.key === "fill" || t.key === "picker" ? 16 : 18,
              border: "none",
              borderRadius: 6,
              background: tool === t.key ? "var(--accent, #e8960a)" : "transparent",
              color: tool === t.key ? "#fff" : "var(--text-secondary, #6b6b6b)",
              cursor: "pointer",
              transition: "background 0.1s",
            }}
          >
            {t.icon}
          </button>
        ))}
      </div>

      <div style={{ width: 1, height: 24, background: "var(--border-light, #efefeb)" }} />

      {/* 预设颜色 */}
      <div style={{ display: "flex", gap: 3, alignItems: "center" }}>
        {PRESET_COLORS.map((c) => (
          <button
            key={c}
            onClick={() => setPixelColor(c)}
            title={c}
            style={{
              width: 22,
              height: 22,
              borderRadius: 4,
              background: c,
              border: color === c ? "2px solid var(--accent, #e8960a)" : "1px solid var(--border, #d4d4ce)",
              cursor: "pointer",
              outline: "none",
              boxShadow: color === c ? "0 0 0 2px rgba(232,150,10,0.3)" : "none",
            }}
          />
        ))}
        {/* 自定义颜色 */}
        <input
          type="color"
          value={color}
          onChange={(e) => setPixelColor(e.target.value)}
          title="自定义颜色"
          style={{
            width: 22,
            height: 22,
            border: "1px solid var(--border, #d4d4ce)",
            borderRadius: 4,
            cursor: "pointer",
            padding: 0,
            background: "none",
          }}
        />
      </div>

      <div style={{ width: 1, height: 24, background: "var(--border-light, #efefeb)" }} />

      {/* 网格大小 */}
      <div style={{ display: "flex", gap: 2, alignItems: "center" }}>
        {CELL_SIZES.map((s) => (
          <button
            key={s}
            onClick={() => setPixelCellSize(s)}
            style={{
              height: 28,
              padding: "0 8px",
              fontSize: 11,
              fontWeight: cellSize === s ? 600 : 400,
              border: "none",
              borderRadius: 4,
              background: cellSize === s ? "var(--accent, #e8960a)" : "transparent",
              color: cellSize === s ? "#fff" : "var(--text-tertiary, #b0b0b0)",
              cursor: "pointer",
              fontFamily: "var(--font-mono)",
            }}
          >
            {s}px
          </button>
        ))}
      </div>

      <div style={{ width: 1, height: 24, background: "var(--border-light, #efefeb)" }} />

      {/* 操作按钮 */}
      <div style={{ display: "flex", gap: 4 }}>
        <button
          onClick={pixelUndo}
          disabled={undoStack.length === 0}
          title="撤销"
          style={{
            height: 32,
            padding: "0 10px",
            fontSize: 12,
            border: "none",
            borderRadius: 6,
            background: "transparent",
            color: undoStack.length > 0 ? "var(--text-secondary, #6b6b6b)" : "var(--text-tertiary, #ccc)",
            cursor: undoStack.length > 0 ? "pointer" : "default",
            opacity: undoStack.length > 0 ? 1 : 0.4,
          }}
        >
          ↩ 撤销
        </button>
        <button
          onClick={clearPixelData}
          title="清空"
          style={{
            height: 32,
            padding: "0 10px",
            fontSize: 12,
            border: "none",
            borderRadius: 6,
            background: "transparent",
            color: "var(--text-secondary, #6b6b6b)",
            cursor: "pointer",
          }}
        >
          ✕ 清空
        </button>
      </div>

      {/* 状态信息 */}
      <span style={{
        fontSize: 10,
        color: "var(--text-tertiary, #b0b0b0)",
        fontFamily: "var(--font-mono)",
        whiteSpace: "nowrap",
      }}>
        {cols}×{rows}
      </span>
    </div>
  );
}
