import type { AppStatus } from "../../store/types";

const COLORS: Record<AppStatus, string> = {
  idle: "#4ade80",
  listening: "#facc15",
  thinking: "#60a5fa",
  executing: "#a78bfa",
  error: "#e94560",
};

const LABELS: Record<AppStatus, string> = {
  idle: "就绪",
  listening: "聆听",
  thinking: "思考",
  executing: "执行",
  error: "错误",
};

export default function StatusLight({ status }: { status: AppStatus }) {
  return (
    <span style={{
      display: "inline-flex",
      alignItems: "center",
      gap: 6,
      fontSize: 12,
      color: COLORS[status],
    }}>
      <span style={{
        display: "inline-block",
        width: 8,
        height: 8,
        borderRadius: "50%",
        background: COLORS[status],
        boxShadow: `0 0 6px ${COLORS[status]}`,
        animation: status !== "idle" ? "pulse 1.5s ease-in-out infinite" : undefined,
      }} />
      {LABELS[status]}
    </span>
  );
}
