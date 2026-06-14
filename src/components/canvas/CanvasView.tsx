import { useEffect, useRef } from "react";
import * as fabric from "fabric";
import { invoke } from "@tauri-apps/api/core";
import { initFabricCanvas } from "../../lib/fabric-setup";
import type { CanvasState, StyleTransferResult } from "../../store/types";
import { useAppStore } from "../../store";
import {
  renderBasicShape,
  renderCompositeShape,
  isCompositeShape,
  isBasicShape,
} from "./ShapeRenderer";

interface CanvasViewProps {
  canvasState: CanvasState | null;
}

/** 默认网格参数 */
const DEFAULT_GRID = {
  grid_size: 20,
  grid_origin_x: 40,
  grid_origin_y: 24,
};

export default function CanvasView({ canvasState }: CanvasViewProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const fabricRef = useRef<fabric.Canvas | null>(null);
  const tooltipRef = useRef<HTMLDivElement>(null);

  // 初始化 Fabric.js + 渲染网格 + 自适应窗口
  useEffect(() => {
    if (!canvasRef.current || !containerRef.current) return;

    // 用窗口尺寸初始化（比 getBoundingClientRect 更可靠）
    const w = window.innerWidth;
    const h = window.innerHeight;

    const { canvas, cleanup } = initFabricCanvas(canvasRef.current, w, h);
    fabricRef.current = canvas;
    renderGrid(canvas, w, h);

    // ResizeObserver：容器大小变化时更新画布和网格
    let firstResize = true;
    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width: cw, height: ch } = entry.contentRect;
        if (!canvas || cw === 0 || ch === 0) return;

        // 首次触发时容器已有真实尺寸，覆盖初始化尺寸
        if (firstResize) {
          firstResize = false;
          if (Math.abs(cw - w) < 20 && Math.abs(ch - h) < 20) return;
        }

        canvas.setWidth(cw);
        canvas.setHeight(ch);
        canvas.calcOffset();

        // 移除旧网格线，重新绘制
        const toRemove: fabric.Object[] = [];
        canvas.getObjects().forEach((obj) => {
          if ((obj as any).isGridLine) toRemove.push(obj);
        });
        toRemove.forEach((obj) => canvas.remove(obj));
        renderGrid(canvas, cw, ch);
        canvas.requestRenderAll();
      }
    });

    observer.observe(containerRef.current);

    // 鼠标移动时显示网格坐标 tooltip
    canvas.on("mouse:move", (opt) => {
      const tip = tooltipRef.current;
      if (!tip) return;

      const pointer = canvas.getScenePoint(opt.e);
      const gx = Math.round((pointer.x - DEFAULT_GRID.grid_origin_x) / DEFAULT_GRID.grid_size);
      const gy = Math.round((pointer.y - DEFAULT_GRID.grid_origin_y) / DEFAULT_GRID.grid_size);

      if (gx >= 0 && gy >= 0) {
        tip.style.display = "block";
        tip.style.left = `${(opt.e as MouseEvent).clientX + 14}px`;
        tip.style.top = `${(opt.e as MouseEvent).clientY - 28}px`;
        tip.textContent = `x:${gx}, y:${gy}`;
      } else {
        tip.style.display = "none";
      }
    });

    canvas.on("mouse:out", () => {
      const tip = tooltipRef.current;
      if (tip) tip.style.display = "none";
    });

    return () => {
      observer.disconnect();
      cleanup();
    };
  }, []);

  // 当 canvasState 变化时渲染节点和连线（网格保留不动）
  const prevStateRef = useRef<CanvasState | null>(null);
  useEffect(() => {
    const canvas = fabricRef.current;
    if (!canvas || !canvasState) return;

    // 判断是否需要全量重绘
    const prev = prevStateRef.current;
    const needsFullRender =
      !prev ||
      prev.nodes !== canvasState.nodes ||
      prev.edges !== canvasState.edges ||
      prev.theme !== canvasState.theme;

    if (needsFullRender) {
      prevStateRef.current = canvasState;
      renderCanvasState(canvas, canvasState);
    }
  }, [canvasState]);

  // 检测 pendingAction（如风格转换），执行前端侧操作
  const pendingAction = useAppStore((s) => s.pendingAction);
  const clearPendingAction = useAppStore((s) => s.clearPendingAction);
  const setStatus = useAppStore((s) => s.setStatus);
  const pendingActionRef = useRef(false);

  useEffect(() => {
    const canvas = fabricRef.current;
    if (!canvas || !pendingAction || pendingActionRef.current) return;

    if (pendingAction.action_type !== "apply_style") return;

    pendingActionRef.current = true;

    const isNodeMode = pendingAction.node_ids.length > 0;
    const nodeIds = pendingAction.node_ids;

    // Async IIFE: cropCanvasRegion is Promise-based
    (async () => {
      try {
        // Step 1: 确定目标包围盒 + 待移除的对象
        let targetBounds = { left: 40, top: 24, width: 200, height: 200 };
        let foundBounds = false;
        const toRemove: fabric.Object[] = [];

        if (isNodeMode) {
          let minX = Infinity,
            minY = Infinity,
            maxX = -Infinity,
            maxY = -Infinity;
          canvas.getObjects().forEach((obj) => {
            const data = (obj as any).data;
            if (data?.nodeId && nodeIds.includes(data.nodeId)) {
              const bounds = obj.getBoundingRect();
              minX = Math.min(minX, bounds.left);
              minY = Math.min(minY, bounds.top);
              maxX = Math.max(maxX, bounds.left + bounds.width);
              maxY = Math.max(maxY, bounds.top + bounds.height);
              toRemove.push(obj);
            }
          });
          if (isFinite(minX)) {
            targetBounds = {
              left: minX - 16,
              top: minY - 8,
              width: maxX - minX + 32,
              height: maxY - minY + 16,
            };
            foundBounds = true;
          }
        } else {
          let minX = Infinity,
            minY = Infinity,
            maxX = -Infinity,
            maxY = -Infinity;
          canvas.getObjects().forEach((obj) => {
            if ((obj as any).isGridLine) return;
            const bounds = obj.getBoundingRect();
            minX = Math.min(minX, bounds.left);
            minY = Math.min(minY, bounds.top);
            maxX = Math.max(maxX, bounds.left + bounds.width);
            maxY = Math.max(maxY, bounds.top + bounds.height);
            toRemove.push(obj);
          });
          if (isFinite(minX)) {
            targetBounds = {
              left: minX,
              top: minY,
              width: maxX - minX,
              height: maxY - minY,
            };
            foundBounds = true;
          }
        }

        // Step 2: 获取图像 — 节点模式裁剪，画布模式全图
        const imageDataUrl =
          isNodeMode && foundBounds
            ? await cropCanvasRegion(canvas, targetBounds)
            : canvas.toDataURL({ format: "png", multiplier: 2 });

        const logLabel = isNodeMode
          ? `${nodeIds.length} 个节点 (${Math.round(targetBounds.width)}×${Math.round(targetBounds.height)}px)`
          : "整个画布";
        logStyleTransfer(`正在调用 API (${logLabel})...`, pendingAction.prompt);

        // 节点模式：prompt 追加作用域约束
        const prompt = isNodeMode
          ? `${pendingAction.prompt}\n仅对选中对象进行风格转换。禁止修改未选中区域。禁止改变画布背景。禁止新增任何元素。严格保持对象轮廓和比例不变。`
          : pendingAction.prompt;

        // Step 3: 调用 Tauri command
        const result = await invoke<StyleTransferResult>(
          "apply_style_transfer",
          { imageBase64: imageDataUrl, prompt, nodeIds },
        );

        logStyleTransfer("风格转换完成，应用结果到画布...");

        // Step 4: 移除旧对象
        toRemove.forEach((obj) => canvas.remove(obj));

        // Step 5: 贴回结果图片
        const img = await fabric.FabricImage.fromURL(result.image_base64);
        if (img) {
          const imgW = img.width!;
          const imgH = img.height!;
          const scaleX = targetBounds.width / imgW;
          const scaleY = targetBounds.height / imgH;
          const scale = foundBounds ? Math.min(scaleX, scaleY) : 1;

          img.set({
            left:
              targetBounds.left +
              (targetBounds.width - imgW * scale) / 2,
            top:
              targetBounds.top +
              (targetBounds.height - imgH * scale) / 2,
            scaleX: scale,
            scaleY: scale,
            selectable: true,
            evented: true,
          });
          (img as any).data = {
            nodeId: `styled_${Date.now()}`,
            isStyledImage: true,
          };
          canvas.add(img);
          canvas.requestRenderAll();
          logStyleTransfer("风格转换完成！");
        }

        clearPendingAction();
        setStatus("idle");
        pendingActionRef.current = false;
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        logStyleTransfer(`风格转换失败: ${msg}`);
        clearPendingAction();
        setStatus("error");
        pendingActionRef.current = false;
        setTimeout(() => setStatus("idle"), 3000);
      }
    })();
  }, [pendingAction, clearPendingAction, setStatus]);

  return (
    <div
      ref={containerRef}
      style={{ flex: 1, overflow: "hidden", position: "relative" }}
    >
      <canvas ref={canvasRef} />
      <div
        ref={tooltipRef}
        style={{
          display: "none",
          position: "fixed",
          pointerEvents: "none",
          background: "var(--surface, #fff)",
          color: "var(--text-primary, #141414)",
          fontSize: 13,
          fontFamily: "var(--font-mono, monospace)",
          fontWeight: 300,
          padding: "4px 10px",
          border: "1px solid var(--border, #e2e2de)",
          borderRadius: "var(--radius, 0)",
          whiteSpace: "nowrap",
          zIndex: 9999,
          lineHeight: 1.6,
          letterSpacing: "0.02em",
        }}
      />
    </div>
  );
}

/** 渲染坐标网格背景 — draw.io 风格淡灰线条 */
function renderGrid(
  canvas: fabric.Canvas,
  width: number,
  height: number,
): void {
  const { grid_size, grid_origin_x, grid_origin_y } = DEFAULT_GRID;

  const offscreen = document.createElement("canvas");
  offscreen.width = width;
  offscreen.height = height;
  const ctx = offscreen.getContext("2d")!;

  // 细线（每格一条，极淡灰）
  ctx.strokeStyle = "#e6e6e2";
  ctx.lineWidth = 0.5;
  ctx.beginPath();
  for (let x = grid_origin_x; x <= width; x += grid_size) {
    ctx.moveTo(x, 0);
    ctx.lineTo(x, height);
  }
  for (let y = grid_origin_y; y <= height; y += grid_size) {
    ctx.moveTo(0, y);
    ctx.lineTo(width, y);
  }
  ctx.stroke();

  // 粗线（每 5 格一条，淡灰）
  const majorStep = grid_size * 5;
  ctx.strokeStyle = "#d4d4ce";
  ctx.lineWidth = 1;
  ctx.beginPath();
  for (let x = grid_origin_x; x <= width; x += majorStep) {
    ctx.moveTo(x, 0);
    ctx.lineTo(x, height);
  }
  for (let y = grid_origin_y; y <= height; y += majorStep) {
    ctx.moveTo(0, y);
    ctx.lineTo(width, y);
  }
  ctx.stroke();

  fabric.FabricImage.fromURL(offscreen.toDataURL()).then((img) => {
    img.set({
      left: 0,
      top: 0,
      selectable: false,
      evented: false,
      excludeFromExport: true,
    });
    (img as any).isGridLine = true;
    canvas.add(img);
    canvas.sendObjectToBack(img);
    canvas.requestRenderAll();
  });
}

/** 将 CanvasState 渲染到 Fabric.js（不清除网格线） */
function renderCanvasState(
  canvas: fabric.Canvas,
  state: CanvasState,
): void {
  // 移除非网格对象（节点、连线、标签）
  const toRemove: fabric.Object[] = [];
  canvas.getObjects().forEach((obj) => {
    if (!(obj as any).isGridLine) {
      toRemove.push(obj);
    }
  });
  toRemove.forEach((obj) => canvas.remove(obj));

  canvas.backgroundColor =
    state.theme === "Dark" ? "#1c1c18" : "#f4f4f2";

  // 渲染连线
  for (const edge of Object.values(state.edges)) {
    renderEdge(canvas, edge, state.nodes);
  }

  // 渲染节点
  for (const node of Object.values(state.nodes)) {
    renderNode(canvas, node);
  }

  canvas.requestRenderAll();
}

/** 渲染单个节点 */
function renderNode(
  canvas: fabric.Canvas,
  node: CanvasState["nodes"][string],
): void {
  const { position, size, style, label, node_type, shape_type } = node;

  // --- Composite shapes: render as fabric.Group (label handled inside ShapeRenderer) ---
  if (shape_type && isCompositeShape(shape_type)) {
    const group = renderCompositeShape(node);
    (group as any).data = { nodeId: node.id, nodeType: node_type, shapeType: shape_type };
    canvas.add(group);
    return;
  }

  // --- Basic geometric shapes: single fabric object + label ---
  if (shape_type && isBasicShape(shape_type)) {
    const shape = renderBasicShape(node);
    const text = new fabric.Text(label, {
      left: position.x + size.width / 2,
      top: position.y + size.height / 2,
      fontSize: style.font_size,
      fontFamily: style.font_family,
      fill: "#1a1a1a",
      originX: "center",
      originY: "center",
      textAlign: "center",
    });

    const group = new fabric.Group([shape, text], {
      left: position.x,
      top: position.y,
    });
    (group as any).data = { nodeId: node.id, nodeType: node_type, shapeType: shape_type };
    canvas.add(group);
    return;
  }

  // --- Existing flowchart node rendering (unchanged) ---
  let shape: fabric.Object;

  switch (node_type) {
    case "Start":
    case "End":
      // 用 Ellipse 画真正的椭圆/圆形（符合流程图规范）
      shape = new fabric.Ellipse({
        left: position.x + size.width / 2,
        top: position.y + size.height / 2,
        rx: size.width / 2,
        ry: size.height / 2,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
        originX: "center",
        originY: "center",
      });
      break;

    case "Decision":
      // 用 Polygon 画真正的菱形（四个顶点），而非旋转矩形
      shape = new fabric.Polygon(
        [
          { x: size.width / 2, y: 0 },
          { x: size.width, y: size.height / 2 },
          { x: size.width / 2, y: size.height },
          { x: 0, y: size.height / 2 },
        ],
        {
          left: position.x,
          top: position.y,
          fill: style.fill,
          stroke: style.stroke,
          strokeWidth: style.stroke_width,
        },
      );
      break;

    case "Data":
      shape = new fabric.Rect({
        left: position.x,
        top: position.y,
        width: size.width,
        height: size.height,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
        skewX: 15,
      });
      break;

    default:
      shape = new fabric.Rect({
        left: position.x,
        top: position.y,
        width: size.width,
        height: size.height,
        rx: style.border_radius,
        ry: style.border_radius,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
      });
  }

  const text = new fabric.Text(label, {
    left: position.x + size.width / 2,
    top: position.y + size.height / 2,
    fontSize: style.font_size,
    fontFamily: style.font_family,
    fill: "#1a1a1a",
    originX: "center",
    originY: "center",
    textAlign: "center",
  });

  const group = new fabric.Group([shape, text], {
    left: position.x,
    top: position.y,
  });
  (group as any).data = { nodeId: node.id, nodeType: node_type };

  canvas.add(group);
}

/** 渲染单条连线（含箭头） */
function renderEdge(
  canvas: fabric.Canvas,
  edge: CanvasState["edges"][string],
  nodes: CanvasState["nodes"],
): void {
  const fromNode = nodes[edge.from_id];
  const toNode = nodes[edge.to_id];
  if (!fromNode || !toNode) return;

  // 计算两节点中心
  const fromCX = fromNode.position.x + fromNode.size.width / 2;
  const fromCY = fromNode.position.y + fromNode.size.height / 2;
  const toCX = toNode.position.x + toNode.size.width / 2;
  const toCY = toNode.position.y + toNode.size.height / 2;

  const dx = toCX - fromCX;
  const dy = toCY - fromCY;

  // 根据两节点相对位置选择最佳的连接点（边的中点）
  let fromX: number, fromY: number, toX: number, toY: number;

  if (Math.abs(dx) > Math.abs(dy)) {
    // 水平方向为主：从左节点的右边 → 右节点的左边
    if (dx > 0) {
      fromX = fromNode.position.x + fromNode.size.width;
      fromY = fromCY;
      toX = toNode.position.x;
      toY = toCY;
    } else {
      fromX = fromNode.position.x;
      fromY = fromCY;
      toX = toNode.position.x + toNode.size.width;
      toY = toCY;
    }
  } else {
    // 垂直方向为主：从上节点的下边 → 下节点的上边
    if (dy > 0) {
      fromX = fromCX;
      fromY = fromNode.position.y + fromNode.size.height;
      toX = toCX;
      toY = toNode.position.y;
    } else {
      fromX = fromCX;
      fromY = fromNode.position.y;
      toX = toCX;
      toY = toNode.position.y + toNode.size.height;
    }
  }

  const dashArray =
    edge.style.line_style === "Dashed"
      ? [8, 4]
      : edge.style.line_style === "Dotted"
        ? [2, 4]
        : undefined;

  const line = new fabric.Line([fromX, fromY, toX, toY], {
    stroke: edge.style.stroke,
    strokeWidth: edge.style.stroke_width,
    strokeDashArray: dashArray,
  });
  (line as any).data = { edgeId: edge.id };
  canvas.add(line);

  // 箭头（三角形）
  const angle = Math.atan2(toY - fromY, toX - fromX);
  const arrowSize = 10;
  const arrow = new fabric.Triangle({
    left: toX,
    top: toY,
    width: arrowSize * 2,
    height: arrowSize * 1.6,
    fill: edge.style.stroke,
    angle: (angle * 180) / Math.PI + 90,
    originX: "center",
    originY: "center",
    selectable: false,
    evented: false,
  });
  (arrow as any).data = { edgeId: edge.id, isArrow: true };
  canvas.add(arrow);

  if (edge.label) {
    const label = new fabric.Text(edge.label, {
      left: (fromX + toX) / 2,
      top: (fromY + toY) / 2 - 16,
      fontSize: 12,
      fill: "#666",
      backgroundColor: "#fafafa",
    });
    canvas.add(label);
  }
}

/** 风格转换流程日志（开发调试用） */
function logStyleTransfer(message: string, detail?: string) {
  const prefix = "[风格转换]";
  if (detail) {
    console.log(`${prefix} ${message}`, detail);
  } else {
    console.log(`${prefix} ${message}`);
  }
}

/** DashScope API 最小尺寸要求 */
const MIN_API_IMAGE_SIZE = 512;

/** 从 Fabric.js canvas 中裁剪指定区域为独立 PNG（base64 data URL）。
 *  小图自动等比放大至不低于 512px，满足 DashScope API 最低尺寸。 */
function cropCanvasRegion(
  canvas: fabric.Canvas,
  bounds: { left: number; top: number; width: number; height: number },
): Promise<string> {
  const fullDataUrl = canvas.toDataURL({
    format: "png",
    multiplier: 2,
  });

  return new Promise((resolve) => {
    const img = new Image();
    img.onload = () => {
      const w = Math.round(bounds.width);
      const h = Math.round(bounds.height);

      // 如果尺寸小于 API 要求，等比放大
      const minDim = Math.min(w, h);
      const scale = minDim < MIN_API_IMAGE_SIZE
        ? MIN_API_IMAGE_SIZE / minDim
        : 1;

      const offscreen = document.createElement("canvas");
      offscreen.width = Math.round(w * scale);
      offscreen.height = Math.round(h * scale);
      const ctx = offscreen.getContext("2d")!;

      // 高质量缩放
      ctx.imageSmoothingEnabled = true;
      ctx.imageSmoothingQuality = "high";
      ctx.drawImage(
        img,
        bounds.left,
        bounds.top,
        bounds.width,
        bounds.height,
        0,
        0,
        offscreen.width,
        offscreen.height,
      );
      resolve(offscreen.toDataURL("image/png"));
    };
    img.src = fullDataUrl;
  });
}
