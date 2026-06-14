import * as fabric from "fabric";
import type { DiagramNode, SubShape } from "../../store/types";

/**
 * 渲染基本几何图形，返回 fabric.Object
 */
export function renderBasicShape(
  node: DiagramNode,
): fabric.Object {
  const { position, size, style, shape_type } = node;
  const cx = position.x + size.width / 2;
  const cy = position.y + size.height / 2;

  switch (shape_type) {
    case "Circle": {
      const rx = size.width / 2;
      const ry = size.height / 2;
      return new fabric.Ellipse({
        left: cx,
        top: cy,
        rx,
        ry,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
        originX: "center",
        originY: "center",
      });
    }

    case "Rectangle": {
      return new fabric.Rect({
        left: position.x,
        top: position.y,
        width: size.width,
        height: size.height,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
        rx: style.border_radius,
        ry: style.border_radius,
      });
    }

    case "Triangle": {
      return new fabric.Triangle({
        left: cx,
        top: cy,
        width: size.width,
        height: size.height,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
        originX: "center",
        originY: "center",
      });
    }

    case "Line": {
      return new fabric.Line(
        [position.x, position.y, position.x + size.width, position.y + size.height],
        {
          stroke: style.stroke || "#333333",
          strokeWidth: style.stroke_width || 2,
        },
      );
    }

    case "Dot": {
      const r = style.stroke_width > 0 ? style.stroke_width * 3 : 8;
      return new fabric.Ellipse({
        left: position.x,
        top: position.y,
        rx: r,
        ry: r,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: 0.5,
        originX: "center",
        originY: "center",
      });
    }

    default:
      return new fabric.Rect({
        left: position.x,
        top: position.y,
        width: size.width,
        height: size.height,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
      });
  }
}

/**
 * 渲染复合图形，返回 fabric.Group（含 label）
 */
export function renderCompositeShape(
  node: DiagramNode,
): fabric.Group {
  const { position, size, style, label, sub_shapes: subShapes } = node;
  const objects: fabric.Object[] = [];

  if (!subShapes || subShapes.length === 0) {
    return new fabric.Group([
      new fabric.Rect({
        width: size.width,
        height: size.height,
        fill: style.fill,
        stroke: style.stroke,
        strokeWidth: style.stroke_width,
      }),
    ]);
  }

  // 子组件使用绝对画布坐标
  for (const sub of subShapes) {
    const obj = renderSubShape(sub);
    if (obj) {
      obj.set({
        left: position.x + sub.rel_x,
        top: position.y + sub.rel_y,
      });
      objects.push(obj);
    }
  }

  // label 放在图形上方中央
  if (label && label.length > 0) {
    const effectiveTextColor = style.text_color || "#1a1a1a";
    const text = new fabric.Textbox(label, {
      left: position.x + size.width / 2,
      top: position.y - 24,
      width: size.width + 16,
      fontSize: style.font_size,
      fontFamily: style.font_family,
      fill: effectiveTextColor,
      originX: "center",
      originY: "bottom",
      textAlign: "center",
      splitByGrapheme: true,
    });
    objects.push(text);
  }

  // Fabric 根据所有对象（含 label）的绝对坐标自动计算 Group 位置和包围盒
  return new fabric.Group(objects);
}

/** 渲染单个子组件 */
function renderSubShape(sub: SubShape): fabric.Object | null {
  switch (sub.shape_type) {
    case "rect": {
      return new fabric.Rect({
        width: sub.width,
        height: sub.height,
        fill: sub.fill,
        stroke: sub.stroke,
        strokeWidth: sub.stroke_width,
      });
    }

    case "circle": {
      const r = sub.radius ?? Math.min(sub.width, sub.height) / 2;
      return new fabric.Ellipse({
        left: 0,
        top: 0,
        rx: r,
        ry: r,
        fill: sub.fill,
        stroke: sub.stroke,
        strokeWidth: sub.stroke_width,
      });
    }

    case "triangle": {
      return new fabric.Triangle({
        width: sub.width,
        height: sub.height,
        fill: sub.fill,
        stroke: sub.stroke,
        strokeWidth: sub.stroke_width,
      });
    }

    case "line": {
      // 从 bounding box 左上角到右下角，覆盖所有方向的射线
      return new fabric.Line(
        [0, 0, sub.width, sub.height],
        {
          stroke: sub.stroke,
          strokeWidth: sub.stroke_width,
        },
      );
    }

    case "arc": {
      return new fabric.Ellipse({
        left: 0,
        top: 0,
        rx: sub.width / 2,
        ry: sub.height,
        fill: sub.fill,
        stroke: sub.stroke,
        strokeWidth: sub.stroke_width,
      });
    }

    case "star_polygon": {
      return createStarPolygon(sub);
    }

    default:
      return new fabric.Rect({
        width: sub.width,
        height: sub.height,
        fill: sub.fill,
        stroke: sub.stroke,
        strokeWidth: sub.stroke_width,
      });
  }
}

/** 五角星多边形：外半径 + 内半径交替顶点 */
function createStarPolygon(sub: SubShape): fabric.Polygon {
  const cx = sub.width / 2;
  const cy = sub.height / 2;
  const outerR = sub.width / 2;
  const innerR = outerR * 0.4;
  const points = 5;
  const vertices: { x: number; y: number }[] = [];

  for (let i = 0; i < points * 2; i++) {
    const angle = (i * Math.PI) / points - Math.PI / 2;
    const r = i % 2 === 0 ? outerR : innerR;
    vertices.push({
      x: cx + r * Math.cos(angle),
      y: cy + r * Math.sin(angle),
    });
  }

  return new fabric.Polygon(vertices, {
    left: 0,
    top: 0,
    fill: sub.fill,
    stroke: sub.stroke,
    strokeWidth: sub.stroke_width,
  });
}

/** 判断 shape_type 是否为复合图形 */
export function isCompositeShape(shapeType: string): boolean {
  return [
    "House", "Sun", "Tree", "Smiley", "Star",
    "Cake", "Gift", "Balloon", "Candle", "Heart",
    "Flower", "ArrowShape", "SpeechBubble", "Cloud", "Lightning",
  ].includes(shapeType);
}

/** 判断 shape_type 是否为基本几何图形 */
export function isBasicShape(shapeType: string): boolean {
  return ["Circle", "Rectangle", "Triangle", "Line", "Dot"].includes(shapeType);
}
