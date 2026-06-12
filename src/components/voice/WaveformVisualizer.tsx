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
      // 清空画布
      const ctx = canvasRef.current?.getContext("2d");
      if (ctx && canvasRef.current) {
        ctx.clearRect(0, 0, canvasRef.current.width, canvasRef.current.height);
      }
      return;
    }

    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const animate = () => {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      const barWidth = canvas.width / barCount;
      for (let i = 0; i < barCount; i++) {
        // 模拟随机音量（正式版接入 Web Audio API analyser）
        const height = isActive
          ? Math.random() * canvas.height * 0.8 + canvas.height * 0.1
          : 3;
        ctx.fillStyle = `hsl(${220 + i * 3}, 80%, ${50 + height * 0.5}%)`;
        ctx.fillRect(
          i * barWidth + 1,
          canvas.height - height,
          barWidth - 2,
          height
        );
      }
      animationRef.current = requestAnimationFrame(animate);
    };
    animate();

    return () => cancelAnimationFrame(animationRef.current);
  }, [isActive, barCount]);

  return (
    <canvas
      ref={canvasRef}
      width={200}
      height={30}
      style={{ borderRadius: 4, flexShrink: 0 }}
    />
  );
}
