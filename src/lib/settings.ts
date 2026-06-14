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
  anthropic: {
    id: "anthropic",
    name: "Anthropic Claude",
    api_key: "",
    endpoint: "https://api.anthropic.com",
    model: "claude-sonnet-4-20250514",
  },
  google: {
    id: "google",
    name: "Google Gemini",
    api_key: "",
    endpoint: "https://generativelanguage.googleapis.com",
    model: "gemini-2.5-flash",
  },
  qwen: {
    id: "qwen",
    name: "通义千问 (Qwen)",
    api_key: "",
    endpoint: "https://dashscope.aliyuncs.com/compatible-mode/v1",
    model: "qwen-max",
  },
  zhipu: {
    id: "zhipu",
    name: "智谱 GLM",
    api_key: "",
    endpoint: "https://open.bigmodel.cn/api/paas/v4",
    model: "glm-4",
  },
  moonshot: {
    id: "moonshot",
    name: "月之暗面 Kimi",
    api_key: "",
    endpoint: "https://api.moonshot.cn/v1",
    model: "moonshot-v1-8k",
  },
  xai: {
    id: "xai",
    name: "xAI Grok",
    api_key: "",
    endpoint: "https://api.x.ai/v1",
    model: "grok-2",
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
  flux: {
    id: "flux",
    name: "Black Forest Labs FLUX",
    api_key: "",
    endpoint: "https://api.bfl.ml/v1",
    model: "flux.1-kontext-dev",
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
  replicate: {
    id: "replicate",
    name: "Replicate",
    api_key: "",
    endpoint: "https://api.replicate.com/v1",
    model: "black-forest-labs/flux-schnell",
  },
  leonardo: {
    id: "leonardo",
    name: "Leonardo AI",
    api_key: "",
    endpoint: "https://cloud.leonardo.ai/api/rest/v1",
    model: "leonardo-ai",
  },
  firefly: {
    id: "firefly",
    name: "Adobe Firefly",
    api_key: "",
    endpoint: "https://firefly-api.adobe.io/v2",
    model: "firefly-image-v3",
  },
  midjourney: {
    id: "midjourney",
    name: "Midjourney",
    api_key: "",
    endpoint: "https://api.midjourney.com",
    model: "midjourney-v6",
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

/** 获取当前激活 LLM 厂商的 Endpoint */
export function getLLMEndpoint(): string | undefined {
  const s = loadSettings();
  const provider = s.llm.providers[s.llm.active];
  return provider?.endpoint || undefined;
}

/** 获取当前激活 LLM 厂商的 Model */
export function getLLMModel(): string | undefined {
  const s = loadSettings();
  const provider = s.llm.providers[s.llm.active];
  return provider?.model || undefined;
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
