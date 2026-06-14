import { useState, useEffect } from "react";
import type { AppSettings, ProviderConfig } from "../../store/types";
import { loadSettings, saveSettings } from "../../lib/settings";

interface SettingsPanelProps {
  open: boolean;
  onClose: () => void;
}

// ── 通用输入样式 ──────────────────────────────────────────────

const fieldStyle: React.CSSProperties = {
  width: "100%",
  boxSizing: "border-box",
  padding: "10px 12px",
  fontSize: 14,
  fontFamily: "var(--font-mono)",
  border: "1px solid var(--border, #e2e2de)",
  borderRadius: "var(--radius, 0)",
  background: "var(--bg, #fafafa)",
  color: "var(--text-primary)",
  outline: "none",
};

const selectStyle: React.CSSProperties = {
  ...fieldStyle,
  cursor: "pointer",
  appearance: "none",
};

const labelStyle: React.CSSProperties = {
  display: "block",
  fontSize: 12,
  fontWeight: 500,
  marginBottom: 4,
  color: "var(--text-secondary)",
  textTransform: "uppercase",
  letterSpacing: "0.05em",
};

// ── 厂商配置区块 ──────────────────────────────────────────────

function ProviderSection({
  title,
  desc,
  providers,
  activeId,
  onActiveChange,
  onProviderChange,
}: {
  title: string;
  desc: string;
  providers: ProviderConfig[];
  activeId: string;
  onActiveChange: (id: string) => void;
  onProviderChange: (id: string, patch: Partial<ProviderConfig>) => void;
}) {
  const active = providers.find((p) => p.id === activeId) ?? providers[0];

  return (
    <div style={{ marginBottom: 28 }}>
      <div style={{
        fontSize: 14,
        fontWeight: 600,
        marginBottom: 2,
        color: "var(--text-primary)",
      }}>
        {title}
      </div>
      <div style={{
        fontSize: 12,
        marginBottom: 14,
        color: "var(--text-secondary)",
        lineHeight: 1.5,
      }}>
        {desc}
      </div>

      {/* 厂商选择 */}
      <label style={labelStyle}>厂商</label>
      <div style={{ position: "relative", marginBottom: 14 }}>
        <select
          value={activeId}
          onChange={(e) => onActiveChange(e.target.value)}
          style={selectStyle}
        >
          {providers.map((p) => (
            <option key={p.id} value={p.id}>{p.name}</option>
          ))}
        </select>
        {/* custom dropdown arrow */}
        <span style={{
          position: "absolute",
          right: 12,
          top: "50%",
          transform: "translateY(-50%)",
          pointerEvents: "none",
          fontSize: 10,
          color: "var(--text-secondary)",
        }}>
          ▼
        </span>
      </div>

      {/* API Key */}
      <label style={labelStyle}>API Key</label>
      <input
        type="password"
        value={active.api_key}
        onChange={(e) => onProviderChange(active.id, { api_key: e.target.value })}
        placeholder="sk-..."
        style={{ ...fieldStyle, marginBottom: 14 }}
      />

      {/* Endpoint */}
      <label style={labelStyle}>Endpoint</label>
      <input
        type="text"
        value={active.endpoint}
        onChange={(e) => onProviderChange(active.id, { endpoint: e.target.value })}
        style={{ ...fieldStyle, marginBottom: 14 }}
      />

      {/* Model */}
      <label style={labelStyle}>Model</label>
      <input
        type="text"
        value={active.model}
        onChange={(e) => onProviderChange(active.id, { model: e.target.value })}
        style={fieldStyle}
      />
    </div>
  );
}

// ── 主面板 ────────────────────────────────────────────────────

export default function SettingsPanel({ open, onClose }: SettingsPanelProps) {
  const [settings, setSettings] = useState<AppSettings | null>(null);
  const [saved, setSaved] = useState(false);

  // 打开时加载
  useEffect(() => {
    if (open) {
      setSettings(loadSettings());
      setSaved(false);
    }
  }, [open]);

  if (!open || !settings) return null;

  const llmProviders = Object.values(settings.llm.providers);
  const imgProviders = Object.values(settings.image.providers);

  const updateProvider = (
    group: "llm" | "image",
    providerId: string,
    patch: Partial<ProviderConfig>,
  ) => {
    setSettings((prev) => {
      if (!prev) return prev;
      const groupData = prev[group];
      return {
        ...prev,
        [group]: {
          ...groupData,
          providers: {
            ...groupData.providers,
            [providerId]: { ...groupData.providers[providerId], ...patch },
          },
        },
      };
    });
  };

  const handleSave = () => {
    saveSettings(settings);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  return (
    <>
      <div
        onClick={onClose}
        style={{
          position: "fixed",
          inset: 0,
          zIndex: 200,
          background: "rgba(0,0,0,0.35)",
        }}
      />

      <div
        style={{
          position: "fixed",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          zIndex: 201,
          width: 460,
          maxHeight: "85vh",
          overflowY: "auto",
          background: "var(--surface, #fff)",
          border: "1px solid var(--border, #e2e2de)",
          borderRadius: "var(--radius, 0)",
          padding: 32,
          fontFamily: "var(--font-sans)",
          color: "var(--text-primary)",
        }}
      >
        {/* header */}
        <div style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: 24,
        }}>
          <h2 style={{ margin: 0, fontSize: 18, fontWeight: 500 }}>
            设置
          </h2>
          <button
            onClick={onClose}
            style={{
              background: "none",
              border: "none",
              cursor: "pointer",
              fontSize: 18,
              color: "var(--text-secondary)",
              padding: "4px 8px",
              lineHeight: 1,
            }}
          >
            ✕
          </button>
        </div>

        {/* LLM section */}
        <ProviderSection
          title="LLM 语音理解"
          desc="选择用于解析语音指令的大语言模型厂商"
          providers={llmProviders}
          activeId={settings.llm.active}
          onActiveChange={(id) =>
            setSettings((prev) =>
              prev
                ? { ...prev, llm: { ...prev.llm, active: id } }
                : prev,
            )
          }
          onProviderChange={(id, patch) => updateProvider("llm", id, patch)}
        />

        {/* 分隔线 */}
        <div style={{
          borderTop: "1px solid var(--border, #e2e2de)",
          marginBottom: 24,
        }} />

        {/* Image section */}
        <ProviderSection
          title="图像生成"
          desc="选择用于风格转换的图像生成模型厂商"
          providers={imgProviders}
          activeId={settings.image.active}
          onActiveChange={(id) =>
            setSettings((prev) =>
              prev
                ? { ...prev, image: { ...prev.image, active: id } }
                : prev,
            )
          }
          onProviderChange={(id, patch) => updateProvider("image", id, patch)}
        />

        {/* save */}
        <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
          <button
            onClick={handleSave}
            style={{
              padding: "10px 28px",
              fontSize: 14,
              fontWeight: 500,
              background: "var(--text-primary, #141414)",
              color: "var(--surface, #fff)",
              border: "none",
              borderRadius: "var(--radius, 0)",
              cursor: "pointer",
              letterSpacing: "0.02em",
            }}
          >
            保存
          </button>
          {saved && (
            <span style={{ fontSize: 13, color: "var(--accent, #43a047)" }}>
              已保存 ✓
            </span>
          )}
        </div>

        <p style={{
          margin: "16px 0 0",
          fontSize: 12,
          color: "var(--text-secondary)",
          lineHeight: 1.6,
        }}>
          未填写 API Key 则使用对应的环境变量。
          添加新厂商只需在代码中扩展 provider 列表即可。
        </p>
      </div>
    </>
  );
}
