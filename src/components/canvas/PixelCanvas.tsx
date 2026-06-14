import { useEffect, useRef, useCallback } from "react";
import { useAppStore } from "../../store";

export default function PixelCanvas() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const isDrawing = useRef(false);
  const lastCell = useRef<[number, number] | null>(null);

  const pixel = useAppStore((s) => s.pixel);
  const setPixelCell = useAppStore((s) => s.setPixelCell);
  const setPixelColor = useAppStore((s) => s.setPixelColor);
  const pixelFloodFill = useAppStore((s) => s.pixelFloodFill);

  const { data, color, tool, cellSize, cols, rows } = pixel;

  // 渲染网格
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

    // 填充的格子（关掉抗锯齿，保持像素锐利）
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

  // 坐标 → 格子
  const cellFromEvent = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>): [number, number] | null => {
      const canvas = canvasRef.current;
      if (!canvas) return null;
      const rect = canvas.getBoundingClientRect();
      const sx = canvas.width / rect.width;
      const sy = canvas.height / rect.height;
      const mx = (e.clientX - rect.left) * sx;
      const my = (e.clientY - rect.top) * sy;
      const col = Math.floor(mx / cellSize);
      const row = Math.floor(my / cellSize);
      if (col < 0 || col >= cols || row < 0 || row >= rows) return null;
      return [row, col];
    },
    [cellSize, cols, rows],
  );

  const handleMouseDown = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      const cell = cellFromEvent(e);
      if (!cell) return;
      const [row, col] = cell;
      isDrawing.current = true;
      lastCell.current = cell;

      if (tool === "pencil") {
        setPixelCell(row, col, color);
      } else if (tool === "eraser") {
        setPixelCell(row, col, null);
      } else if (tool === "fill") {
        pixelFloodFill(row, col, color);
      } else if (tool === "picker") {
        const key = `${row},${col}`;
        const picked = data[key] ?? "#1a1a1a";
        setPixelColor(picked);
      }
    },
    [tool, color, data, cellFromEvent, setPixelCell, pixelFloodFill, setPixelColor],
  );

  const handleMouseMove = useCallback(
    (e: React.MouseEvent<HTMLCanvasElement>) => {
      if (!isDrawing.current) return;
      const cell = cellFromEvent(e);
      if (!cell) return;
      if (lastCell.current && lastCell.current[0] === cell[0] && lastCell.current[1] === cell[1]) return;
      lastCell.current = cell;
      const [row, col] = cell;

      if (tool === "pencil") {
        setPixelCell(row, col, color);
      } else if (tool === "eraser") {
        setPixelCell(row, col, null);
      }
      // fill 和 picker 只在 mousedown 触发一次
    },
    [tool, color, cellFromEvent, setPixelCell],
  );

  const handleMouseUp = useCallback(() => {
    isDrawing.current = false;
    lastCell.current = null;
  }, []);

  const canvasW = cols * cellSize;
  const canvasH = rows * cellSize;

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
        width={canvasW}
        height={canvasH}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
        style={{
          cursor: tool === "picker" ? "crosshair" : tool === "fill" ? "pointer" : "cell",
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
