import { useEffect, useRef } from "react";

interface WaveformVisualizerProps {
  isActive: boolean;
  barCount?: number;
}

export default function WaveformVisualizer({ isActive, barCount = 16 }: WaveformVisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number>(0);

  useEffect(() => {
    if (!isActive) {
      cancelAnimationFrame(animationRef.current);
      const ctx = canvasRef.current?.getContext("2d");
      if (ctx && canvasRef.current) {
        ctx.clearRect(0, 0, canvasRef.current.width, canvasRef.current.height);
        // Draw a flat idle line
        const c = canvasRef.current;
        ctx.fillStyle = "var(--border, #e2e2de)";
        ctx.fillRect(0, c.height / 2 - 1, c.width, 2);
      }
      return;
    }

    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const animate = () => {
      const w = canvas.width;
      const h = canvas.height;
      ctx.clearRect(0, 0, w, h);

      const barWidth = w / barCount;

      for (let i = 0; i < barCount; i++) {
        const barH = Math.random() * h * 0.7 + h * 0.15;
        const x = i * barWidth + 1;
        const y = (h - barH) / 2;

        // Monochrome: only accent color with varying opacity
        const alpha = 0.25 + (barH / h) * 0.55;
        ctx.fillStyle = `rgba(232, 150, 10, ${alpha.toFixed(2)})`;
        ctx.fillRect(x, y, barWidth - 2, barH);
      }
      animationRef.current = requestAnimationFrame(animate);
    };
    animate();

    return () => cancelAnimationFrame(animationRef.current);
  }, [isActive, barCount]);

  return (
    <canvas
      ref={canvasRef}
      width={120}
      height={24}
      style={{ flexShrink: 0, display: "block" }}
    />
  );
}
