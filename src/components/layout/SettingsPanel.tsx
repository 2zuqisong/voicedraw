import { useState, useEffect } from "react";

/** localStorage keys */
const DEEPSEEK_KEY = "vtod_deepseek_api_key";
const DASHSCOPE_KEY = "vtod_dashscope_api_key";

interface SettingsPanelProps {
  open: boolean;
  onClose: () => void;
}

/** 获取持久化的 API Key */
export function getStoredApiKeys(): { deepseek: string; dashscope: string } {
  return {
    deepseek: localStorage.getItem(DEEPSEEK_KEY) || "",
    dashscope: localStorage.getItem(DASHSCOPE_KEY) || "",
  };
}

export default function SettingsPanel({ open, onClose }: SettingsPanelProps) {
  const [deepseek, setDeepseek] = useState("");
  const [dashscope, setDashscope] = useState("");
  const [saved, setSaved] = useState(false);

  // 打开时加载已保存的值
  useEffect(() => {
    if (open) {
      setDeepseek(localStorage.getItem(DEEPSEEK_KEY) || "");
      setDashscope(localStorage.getItem(DASHSCOPE_KEY) || "");
      setSaved(false);
    }
  }, [open]);

  const handleSave = () => {
    if (deepseek.trim()) {
      localStorage.setItem(DEEPSEEK_KEY, deepseek.trim());
    } else {
      localStorage.removeItem(DEEPSEEK_KEY);
    }
    if (dashscope.trim()) {
      localStorage.setItem(DASHSCOPE_KEY, dashscope.trim());
    } else {
      localStorage.removeItem(DASHSCOPE_KEY);
    }
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  if (!open) return null;

  return (
    <>
      {/* backdrop */}
      <div
        onClick={onClose}
        style={{
          position: "fixed",
          inset: 0,
          zIndex: 200,
          background: "rgba(0,0,0,0.35)",
        }}
      />

      {/* panel */}
      <div
        style={{
          position: "fixed",
          top: "50%",
          left: "50%",
          transform: "translate(-50%, -50%)",
          zIndex: 201,
          width: 420,
          background: "var(--surface, #fff)",
          border: "1px solid var(--border, #e2e2de)",
          borderRadius: "var(--radius, 0)",
          padding: 32,
          fontFamily: "var(--font-sans)",
          color: "var(--text-primary)",
        }}
      >
        {/* title */}
        <div style={{
          display: "flex",
          justifyContent: "space-between",
          alignItems: "center",
          marginBottom: 24,
        }}>
          <h2 style={{
            margin: 0,
            fontSize: 18,
            fontWeight: 500,
            letterSpacing: "-0.01em",
          }}>
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

        {/* DeepSeek */}
        <label style={{
          display: "block",
          fontSize: 13,
          fontWeight: 500,
          marginBottom: 6,
          color: "var(--text-secondary)",
        }}>
          DeepSeek API Key
        </label>
        <input
          type="password"
          value={deepseek}
          onChange={(e) => setDeepseek(e.target.value)}
          placeholder="sk-..."
          style={{
            width: "100%",
            boxSizing: "border-box",
            padding: "10px 12px",
            marginBottom: 20,
            fontSize: 14,
            fontFamily: "var(--font-mono)",
            border: "1px solid var(--border, #e2e2de)",
            borderRadius: "var(--radius, 0)",
            background: "var(--bg, #fafafa)",
            color: "var(--text-primary)",
            outline: "none",
          }}
        />

        {/* DashScope */}
        <label style={{
          display: "block",
          fontSize: 13,
          fontWeight: 500,
          marginBottom: 6,
          color: "var(--text-secondary)",
        }}>
          通义万相 DashScope API Key
        </label>
        <input
          type="password"
          value={dashscope}
          onChange={(e) => setDashscope(e.target.value)}
          placeholder="sk-..."
          style={{
            width: "100%",
            boxSizing: "border-box",
            padding: "10px 12px",
            marginBottom: 24,
            fontSize: 14,
            fontFamily: "var(--font-mono)",
            border: "1px solid var(--border, #e2e2de)",
            borderRadius: "var(--radius, 0)",
            background: "var(--bg, #fafafa)",
            color: "var(--text-primary)",
            outline: "none",
          }}
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
            <span style={{
              fontSize: 13,
              color: "var(--accent, #43a047)",
            }}>
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
          未填写则使用环境变量（DEEPSEEK_API_KEY / DASHSCOPE_API_KEY）。
        </p>
      </div>
    </>
  );
}
