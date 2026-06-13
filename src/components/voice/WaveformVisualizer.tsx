import { useEffect, useRef } from "react";

interface WaveformVisualizerProps {
  isActive: boolean;
  barCount?: number;
}

export default function WaveformVisualizer({ isActive, barCount = 20 }: WaveformVisualizerProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number>(0);

  useEffect(() => {
    if (!isActive) {
      cancelAnimationFrame(animationRef.current);
      const ctx = canvasRef.current?.getContext("2d");
      if (ctx && canvasRef.current) {
        const c = canvasRef.current;
        ctx.clearRect(0, 0, c.width, c.height);
        // Idle flat line
        ctx.fillStyle = "#d4d4ce";
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
      width={180}
      height={36}
      style={{ flexShrink: 0, display: "block" }}
    />
  );
}
