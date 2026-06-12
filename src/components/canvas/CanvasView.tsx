import { useEffect, useRef } from "react";
import * as fabric from "fabric";
import { initFabricCanvas } from "../../lib/fabric-setup";
import type { CanvasState } from "../../store/types";

interface CanvasViewProps {
  canvasState: CanvasState | null;
}

export default function CanvasView({ canvasState }: CanvasViewProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const fabricRef = useRef<fabric.Canvas | null>(null);

  // 初始化 Fabric.js
  useEffect(() => {
    if (!canvasRef.current) return;
    const { canvas, cleanup } = initFabricCanvas(canvasRef.current);
    fabricRef.current = canvas;
    return cleanup;
  }, []);

  // 当 canvasState 变化时重新渲染
  useEffect(() => {
    if (!fabricRef.current || !canvasState) return;
    renderCanvasState(fabricRef.current, canvasState);
  }, [canvasState]);

  return (
    <div style={{ flex: 1, overflow: "hidden", position: "relative" }}>
      <canvas ref={canvasRef} />
    </div>
  );
}

/** 将 CanvasState 渲染到 Fabric.js */
function renderCanvasState(canvas: fabric.Canvas, state: CanvasState): void {
  canvas.clear();
  canvas.backgroundColor = state.theme === "Dark" ? "#263238" : "#fafafa";

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
function renderNode(canvas: fabric.Canvas, node: CanvasState["nodes"][string]): void {
  const { position, size, style, label, node_type } = node;

  let shape: fabric.Object;

  switch (node_type) {
    case "Start":
    case "End":
      shape = new fabric.Rect({
        left: position.x,
        top: position.y,
        width: size.width,
        height: size.height,
        rx: size.height / 2,  // 圆角矩形（胶囊形）
        ry: size.height / 2,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
      });
      break;

    case "Decision":
      // 菱形：用旋转45度的矩形
      shape = new fabric.Rect({
        left: position.x,
        top: position.y,
        width: size.width,
        height: size.height,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
        angle: 45,
      });
      break;

    case "Data":
      // 平行四边形（用 skewX 近似）
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

    default: // Process, Subprocess, Text
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

  // 添加文字标签
  const text = new fabric.Text(label, {
    left: position.x + size.width / 2,
    top: position.y + size.height / 2,
    fontSize: style.font_size,
    fontFamily: style.font_family,
    fill: node_type === "Decision" ? "#333" : "#333",
    originX: "center",
    originY: "center",
    textAlign: "center",
  });

  const group = new fabric.Group([shape, text], {
    left: position.x,
    top: position.y,
  });
  // fabric 6.x 不支持在构造参数中传 data，需在创建后设置
  (group as any).data = { nodeId: node.id, nodeType: node_type };

  canvas.add(group);
}

/** 渲染单条连线 */
function renderEdge(
  canvas: fabric.Canvas,
  edge: CanvasState["edges"][string],
  nodes: CanvasState["nodes"]
): void {
  const fromNode = nodes[edge.from_id];
  const toNode = nodes[edge.to_id];
  if (!fromNode || !toNode) return;

  const fromX = fromNode.position.x + fromNode.size.width / 2;
  const fromY = fromNode.position.y + fromNode.size.height;
  const toX = toNode.position.x + toNode.size.width / 2;
  const toY = toNode.position.y;

  const dashArray = edge.style.line_style === "Dashed" ? [8, 4]
    : edge.style.line_style === "Dotted" ? [2, 4]
    : undefined;

  const line = new fabric.Line([fromX, fromY, toX, toY], {
    stroke: edge.style.stroke,
    strokeWidth: edge.style.stroke_width,
    strokeDashArray: dashArray,
  });
  (line as any).data = { edgeId: edge.id };

  canvas.add(line);
  canvas.sendObjectToBack(line); // 连线在节点下方

  // 边标签
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