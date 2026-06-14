import type { AppSettings, ProviderConfig } from "../store/types";

const STORAGE_KEY = "vtod_settings";

// ── 默认厂商配置 ──────────────────────────────────────────────

const LLM_PROVIDER_DEFAULTS: Record<string, ProviderConfig> = {
  deepseek: {
    id: "deepseek",
    name: "DeepSeek",
    api_key: "",
    endpoint: "https://api.deepseek.com",
    model: "deepseek-chat",
  },
  openai: {
    id: "openai",
    name: "OpenAI",
    api_key: "",
    endpoint: "https://api.openai.com/v1",
    model: "gpt-4o",
  },
};

const IMAGE_PROVIDER_DEFAULTS: Record<string, ProviderConfig> = {
  dashscope: {
    id: "dashscope",
    name: "通义万相 (DashScope)",
    api_key: "",
    endpoint:
      "https://dashscope.aliyuncs.com/api/v1/services/aigc/image2image/image-synthesis",
    model: "wanx2.1-imageedit",
  },
  stability: {
    id: "stability",
    name: "Stability AI",
    api_key: "",
    endpoint: "https://api.stability.ai/v2beta/stable-image/control/style",
    model: "stable-image",
  },
  openai_image: {
    id: "openai_image",
    name: "OpenAI DALL·E",
    api_key: "",
    endpoint: "https://api.openai.com/v1/images",
    model: "dall-e-3",
  },
};

// ── 默认设置 ──────────────────────────────────────────────────

function buildDefaults(): AppSettings {
  return {
    llm: {
      active: "deepseek",
      providers: { ...LLM_PROVIDER_DEFAULTS },
    },
    image: {
      active: "dashscope",
      providers: { ...IMAGE_PROVIDER_DEFAULTS },
    },
  };
}

// ── 加载 / 保存 ───────────────────────────────────────────────

/** 加载设置（合并默认值，确保新增厂商自动出现） */
export function loadSettings(): AppSettings {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return buildDefaults();

    const saved = JSON.parse(raw) as Partial<AppSettings>;
    const defaults = buildDefaults();

    // 深度合并：保留已保存的值，用默认值填充缺失的 provider
    return {
      llm: mergeGroup(saved.llm, defaults.llm),
      image: mergeGroup(saved.image, defaults.image),
    };
  } catch {
    return buildDefaults();
  }
}

function mergeGroup(
  saved: Partial<AppSettings["llm"]> | undefined,
  defaults: AppSettings["llm"],
): AppSettings["llm"] {
  if (!saved) return defaults;
  return {
    active: saved.active ?? defaults.active,
    providers: {
      ...defaults.providers,
      ...Object.fromEntries(
        Object.entries(saved.providers ?? {}).map(([id, p]) => [
          id,
          { ...defaults.providers[id], ...p } as ProviderConfig,
        ]),
      ),
    },
  };
}

export function saveSettings(s: AppSettings): void {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(s));
}

// ── 便捷访问 ──────────────────────────────────────────────────

/** 获取当前激活 LLM 厂商的 API Key */
export function getLLMApiKey(): string | undefined {
  const s = loadSettings();
  const provider = s.llm.providers[s.llm.active];
  return provider?.api_key || undefined;
}

/** 获取当前激活图像厂商的 API Key */
export function getImageApiKey(): string | undefined {
  const s = loadSettings();
  const provider = s.image.providers[s.image.active];
  return provider?.api_key || undefined;
}

/** 获取所有 LLM 厂商列表（按 ID 顺序） */
export function getLLMProviderList(): ProviderConfig[] {
  return Object.values(loadSettings().llm.providers);
}

/** 获取所有图像厂商列表 */
export function getImageProviderList(): ProviderConfig[] {
  return Object.values(loadSettings().image.providers);
}
