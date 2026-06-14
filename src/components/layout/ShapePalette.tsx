import { useState } from "react";

interface ShapeEntry {
  name: string;
  command: string;
  category: "basic" | "composite";
  icon: React.ReactNode;
}

const SIZE = 20;

/* ── 微型 SVG 图标工厂 ─────────────────────────────── */

function Svg({ children, vb = "0 0 20 20" }: { children: React.ReactNode; vb?: string }) {
  return (
    <svg width={SIZE} height={SIZE} viewBox={vb} fill="none"
      stroke="var(--text-secondary, #6b6b6b)" strokeWidth="1.2"
      strokeLinecap="round" strokeLinejoin="round"
    >
      {children}
    </svg>
  );
}

const ICONS: Record<string, React.ReactNode> = {
  // ── 基础几何 ──
  circle: <Svg><circle cx="10" cy="10" r="7" fill="var(--accent-dim, rgba(232,150,10,0.1))" /></Svg>,
  rectangle: <Svg><rect x="3" y="3" width="14" height="14" rx="1" fill="var(--accent-dim, rgba(232,150,10,0.1))" /></Svg>,
  triangle: <Svg><polygon points="10,2 17,17 3,17" fill="var(--accent-dim, rgba(232,150,10,0.1))" /></Svg>,
  line: <Svg><line x1="3" y1="17" x2="17" y2="3" /></Svg>,
  dot: <Svg><circle cx="10" cy="10" r="3" fill="var(--text-secondary, #6b6b6b)" stroke="none" /></Svg>,

  // ── 复合图形 ──
  house: <Svg><polygon points="10,2 18,10 2,10" fill="var(--accent-dim, rgba(232,150,10,0.1))" /><rect x="4" y="10" width="12" height="8" fill="var(--accent-dim, rgba(232,150,10,0.1))" /><rect x="7" y="13" width="3" height="5" /><rect x="12" y="12" width="2.5" height="2.5" /></Svg>,
  sun: <Svg><circle cx="10" cy="10" r="4" fill="var(--accent-dim, rgba(232,150,10,0.15))" />{[0,45,90,135,180,225,270,315].map(a => { const rx=10+4*Math.cos(a*Math.PI/180); const ry=10+4*Math.sin(a*Math.PI/180); return <line key={a} x1={rx-1.2*Math.cos(a*Math.PI/180)} y1={ry-1.2*Math.sin(a*Math.PI/180)} x2={rx} y2={ry} />; })}</Svg>,
  tree: <Svg><circle cx="10" cy="6" r="5" fill="var(--accent-dim, rgba(232,150,10,0.1))" /><rect x="7.5" y="10" width="5" height="8" /></Svg>,
  smiley: <Svg><circle cx="10" cy="10" r="8" fill="var(--accent-dim, rgba(232,150,10,0.1))" /><circle cx="7" cy="8" r="1" fill="var(--text-secondary)" stroke="none" /><circle cx="13" cy="8" r="1" fill="var(--text-secondary)" stroke="none" /><path d="M6,13 Q10,17 14,13" /></Svg>,
  star: <Svg vb="0 0 24 24"><polygon points="12,2 15,9 22,9 16,14 18,21 12,17 6,21 8,14 2,9 9,9" fill="var(--accent-dim, rgba(232,150,10,0.15))" /></Svg>,

  // ── 新增复合图形 ──
  cake: <Svg><rect x="3" y="10" width="14" height="8" rx="1" fill="var(--accent-dim, rgba(232,150,10,0.1))" /><rect x="5" y="6" width="10" height="5" rx="1" fill="var(--accent-dim, rgba(232,150,10,0.08))" /><line x1="7" y1="2" x2="7" y2="6" /><line x1="10" y1="2" x2="10" y2="6" /><line x1="13" y1="2" x2="13" y2="6" /><circle cx="7" cy="1.2" r="0.8" fill="var(--accent, #e8960a)" stroke="none" /><circle cx="10" cy="1.2" r="0.8" fill="var(--accent, #e8960a)" stroke="none" /><circle cx="13" cy="1.2" r="0.8" fill="var(--accent, #e8960a)" stroke="none" /></Svg>,
  gift: <Svg><rect x="2" y="7" width="16" height="11" rx="1" fill="var(--accent-dim, rgba(232,150,10,0.1))" /><rect x="1" y="4" width="18" height="4" rx="1" /><line x1="10" y1="4" x2="10" y2="18" /><line x1="1" y1="10" x2="19" y2="10" /></Svg>,
  balloon: <Svg><ellipse cx="10" cy="7" rx="6" ry="7" fill="var(--accent-dim, rgba(232,150,10,0.1))" /><polygon points="10,14 8,15.5 12,15.5" fill="var(--text-secondary)" stroke="none" /><line x1="10" y1="15.5" x2="10" y2="19" /></Svg>,
  candle: <Svg><rect x="7" y="8" width="6" height="10" rx="0.5" fill="var(--accent-dim, rgba(232,150,10,0.1))" /><ellipse cx="10" cy="6" rx="3" ry="4" fill="var(--accent, #e8960a)" stroke="none" opacity="0.6" /></Svg>,
  heart: <Svg vb="0 0 24 24"><circle cx="9" cy="9" r="6" fill="var(--accent-dim, rgba(232,150,10,0.1))" /><circle cx="15" cy="9" r="6" fill="var(--accent-dim, rgba(232,150,10,0.1))" /><polygon points="3,11.5 12,21 21,11.5" fill="var(--accent-dim, rgba(232,150,10,0.1))" /></Svg>,
  flower: <Svg><circle cx="10" cy="10" r="3" fill="var(--accent, #e8960a)" stroke="none" opacity="0.5" />{[0,72,144,216,288].map(a => <circle key={a} cx={10+4*Math.cos(a*Math.PI/180)} cy={10+4*Math.sin(a*Math.PI/180)} r="3" fill="var(--accent-dim, rgba(232,150,10,0.1))" />)}</Svg>,
  arrow_shape: <Svg><rect x="2" y="8" width="11" height="4" rx="1" fill="var(--accent-dim, rgba(232,150,10,0.1))" /><polygon points="13,5 19,10 13,15" fill="var(--accent-dim, rgba(232,150,10,0.1))" /></Svg>,
  speech_bubble: <Svg><rect x="2" y="2" width="16" height="11" rx="3" fill="var(--accent-dim, rgba(232,150,10,0.1))" /><polygon points="5,13 7,18 10,13" fill="var(--accent-dim, rgba(232,150,10,0.1))" /></Svg>,
  cloud: <Svg><circle cx="8" cy="11" r="4" fill="var(--accent-dim, rgba(232,150,10,0.1))" /><circle cx="14" cy="9" r="5" fill="var(--accent-dim, rgba(232,150,10,0.1))" /><circle cx="11" cy="7" r="4" fill="var(--accent-dim, rgba(232,150,10,0.1))" /></Svg>,
  lightning: <Svg><polyline points="12,1 5,11 11,11 8,19 16,8 10,8" fill="var(--accent-dim, rgba(232,150,10,0.1))" /></Svg>,
};

/* ── 图形列表 ──────────────────────────────────── */

const SHAPES: ShapeEntry[] = [
  { name: "圆形", command: "画一个圆形", category: "basic", icon: ICONS.circle },
  { name: "矩形", command: "画一个矩形", category: "basic", icon: ICONS.rectangle },
  { name: "三角形", command: "画一个三角形", category: "basic", icon: ICONS.triangle },
  { name: "线段", command: "画一条线", category: "basic", icon: ICONS.line },
  { name: "点", command: "画一个点", category: "basic", icon: ICONS.dot },
  { name: "房子", command: "画一座房子", category: "composite", icon: ICONS.house },
  { name: "太阳", command: "画一个太阳", category: "composite", icon: ICONS.sun },
  { name: "树", command: "画一棵树", category: "composite", icon: ICONS.tree },
  { name: "笑脸", command: "画一个笑脸", category: "composite", icon: ICONS.smiley },
  { name: "星星", command: "画一个五角星", category: "composite", icon: ICONS.star },
  { name: "蛋糕", command: "画一个蛋糕", category: "composite", icon: ICONS.cake },
  { name: "礼物盒", command: "画一个礼物盒", category: "composite", icon: ICONS.gift },
  { name: "气球", command: "画一个气球", category: "composite", icon: ICONS.balloon },
  { name: "蜡烛", command: "画一根蜡烛", category: "composite", icon: ICONS.candle },
  { name: "爱心", command: "画一个爱心", category: "composite", icon: ICONS.heart },
  { name: "花朵", command: "画一朵花", category: "composite", icon: ICONS.flower },
  { name: "箭头", command: "画一个箭头", category: "composite", icon: ICONS.arrow_shape },
  { name: "对话气泡", command: "画一个对话气泡", category: "composite", icon: ICONS.speech_bubble },
  { name: "云朵", command: "画一朵云", category: "composite", icon: ICONS.cloud },
  { name: "闪电", command: "画一道闪电", category: "composite", icon: ICONS.lightning },
];

/* ── 组件 ────────────────────────────────────── */

export default function ShapePalette() {
  const [collapsed, setCollapsed] = useState(false);

  return (
    <div style={{
      position: "absolute",
      left: 12,
      top: "50%",
      transform: "translateY(-50%)",
      maxHeight: "calc(100vh - 200px)",
      zIndex: 80,
      display: "flex",
      flexDirection: "column",
      background: "var(--surface, #fff)",
      border: "1px solid var(--border, #e2e2de)",
      borderRadius: "var(--radius, 8px)",
      boxShadow: "0 2px 16px rgba(0,0,0,0.06)",
      overflow: "hidden",
      transition: "width 0.2s ease",
      width: collapsed ? 36 : 148,
      userSelect: "none",
    }}>
      {/* 头部 / 折叠按钮 */}
      <button
        onClick={() => setCollapsed(!collapsed)}
        title={collapsed ? "展开图形库" : "收起图形库"}
        style={{
          display: "flex",
          alignItems: "center",
          gap: 6,
          padding: collapsed ? "10px 8px" : "10px 12px",
          border: "none",
          borderBottom: collapsed ? "none" : "1px solid var(--border-light, #efefeb)",
          background: "transparent",
          cursor: "pointer",
          fontSize: 12,
          fontWeight: 600,
          color: "var(--text-secondary, #6b6b6b)",
          whiteSpace: "nowrap",
          justifyContent: collapsed ? "center" : "flex-start",
        }}
      >
        <span style={{ fontSize: 14, flexShrink: 0 }}>{collapsed ? "▸" : "◂"}</span>
        {!collapsed && "图形库"}
      </button>

      {/* 图形列表 */}
      <div style={{
        flex: 1,
        overflowY: "auto",
        padding: collapsed ? "4px 0" : "6px 0",
        display: collapsed ? "none" : "block",
      }}>
        {/* 基础图形 */}
        <SectionLabel text="基础图形" />
        {SHAPES.filter(s => s.category === "basic").map(s => (
          <ShapeItem key={s.name} shape={s} />
        ))}

        {/* 分隔线 */}
        <div style={{ margin: "6px 12px", height: 1, background: "var(--border-light, #efefeb)" }} />

        {/* 复合图形 */}
        <SectionLabel text="复合图形" />
        {SHAPES.filter(s => s.category === "composite").map(s => (
          <ShapeItem key={s.name} shape={s} />
        ))}
      </div>
    </div>
  );
}

/* ── 子组件 ──────────────────────────────────── */

function SectionLabel({ text }: { text: string }) {
  return (
    <div style={{
      padding: "4px 12px 2px",
      fontSize: 10,
      fontWeight: 600,
      color: "var(--text-tertiary, #b0b0b0)",
      textTransform: "uppercase",
      letterSpacing: "0.04em",
    }}>
      {text}
    </div>
  );
}

function ShapeItem({ shape }: { shape: ShapeEntry }) {
  return (
    <div
      title={`说"${shape.command}"`}
      style={{
        display: "flex",
        alignItems: "center",
        gap: 8,
        padding: "5px 12px",
        cursor: "default",
        fontSize: 12,
        color: "var(--text-primary, #141414)",
        lineHeight: 1.4,
        transition: "background 0.1s",
      }}
      onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.background = "var(--accent-dim, rgba(232,150,10,0.08))"; }}
      onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.background = "transparent"; }}
    >
      <span style={{ flexShrink: 0, display: "flex", alignItems: "center", justifyContent: "center", width: SIZE, height: SIZE }}>
        {shape.icon}
      </span>
      <span style={{ whiteSpace: "nowrap" }}>{shape.name}</span>
    </div>
  );
}
