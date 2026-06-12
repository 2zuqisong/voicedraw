import * as fabric from "fabric";

export interface CanvasSetup {
  canvas: fabric.Canvas;
  cleanup: () => void;
}

/**
 * 初始化 Fabric.js Canvas 实例
 * @param canvasEl - HTML canvas 元素
 * @param width - 画布宽度
 * @param height - 画布高度
 */
export function initFabricCanvas(
  canvasEl: HTMLCanvasElement,
  width: number = 1200,
  height: number = 800
): CanvasSetup {
  const canvas = new fabric.Canvas(canvasEl, {
    width,
    height,
    backgroundColor: "#fafafa",
    selection: false,           // 禁用框选（纯语音无鼠标交互）
    preserveObjectStacking: true,
    renderOnAddRemove: true,
  });

  // 启用缩放和平移（开发者调试用键盘/触摸板）
  canvas.on("mouse:wheel", (opt) => {
    const delta = opt.e.deltaY;
    let zoom = canvas.getZoom();
    zoom *= 0.999 ** delta;
    zoom = Math.min(Math.max(zoom, 0.2), 5);
    canvas.zoomToPoint(new fabric.Point(opt.e.offsetX, opt.e.offsetY), zoom);
    opt.e.preventDefault();
    opt.e.stopPropagation();
  });

  const cleanup = () => {
    canvas.dispose();
  };

  return { canvas, cleanup };
}