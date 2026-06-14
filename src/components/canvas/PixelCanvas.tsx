import { useEffect, useRef, useCallback } from "react";
import { useAppStore } from "../../store";

/**
 * 像素画布 — 纯渲染组件，只展示 Rust/LLM 生产的像素数据，不支持鼠标绘制。
 */
export default function PixelCanvas() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const pixel = useAppStore((s) => s.pixel);

  const { data, cellSize, cols, rows } = pixel;

  const render = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const w = cols * cellSize;
    const h = rows * cellSize;
    canvas.width = w;
    canvas.height = h;

    // 背景
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, w, h);

    // 填充格
    ctx.imageSmoothingEnabled = false;
    for (const [key, hex] of Object.entries(data)) {
      const [r, c] = key.split(",").map(Number);
      ctx.fillStyle = hex;
      ctx.fillRect(c * cellSize, r * cellSize, cellSize, cellSize);
    }

    // 网格线
    ctx.strokeStyle = "#e0e0e0";
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    for (let r = 0; r <= rows; r++) {
      const y = r * cellSize + 0.5;
      ctx.moveTo(0, y);
      ctx.lineTo(w, y);
    }
    for (let c = 0; c <= cols; c++) {
      const x = c * cellSize + 0.5;
      ctx.moveTo(x, 0);
      ctx.lineTo(x, h);
    }
    ctx.stroke();
  }, [data, cellSize, cols, rows]);

  useEffect(() => {
    render();
  }, [render]);

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        overflow: "auto",
        background: "var(--bg, #f5f5f0)",
      }}
    >
      <canvas
        ref={canvasRef}
        style={{
          boxShadow: "0 2px 20px rgba(0,0,0,0.08)",
          borderRadius: 2,
          maxWidth: "calc(100% - 40px)",
          maxHeight: "calc(100% - 40px)",
          objectFit: "contain",
          imageRendering: "pixelated",
        }}
      />
    </div>
  );
}
