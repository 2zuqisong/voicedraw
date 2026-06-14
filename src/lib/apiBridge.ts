/**
 * API Bridge — 自动检测 Tauri / Browser 环境，路由到正确的通信方式。
 *
 * - Tauri 环境：使用 @tauri-apps/api 的 invoke IPC
 * - 浏览器环境：使用 HTTP fetch 到 http://localhost:1421
 */

const API_BASE = "http://127.0.0.1:1421";

function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI__" in window;
}

// 懒加载 Tauri API（避免浏览器环境下 import 失败）
let _tauriInvoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null = null;
let _tauriListen: ((event: string, handler: (e: { payload: unknown }) => void) => Promise<() => void>) | null = null;
let _tauriLoaded = false;
let _tauriLoadPromise: Promise<void> | null = null;

async function ensureTauri() {
  if (_tauriLoaded) return;
  if (!_tauriLoadPromise) {
    _tauriLoadPromise = (async () => {
      const core = await import("@tauri-apps/api/core");
      const event = await import("@tauri-apps/api/event");
      _tauriInvoke = core.invoke;
      _tauriListen = event.listen;
      _tauriLoaded = true;
    })();
  }
  await _tauriLoadPromise;
}

/**
 * 调用后端命令（兼容 Tauri invoke 和浏览器 HTTP）。
 */
export async function invoke<T = unknown>(
  cmd: string,
  args?: Record<string, unknown>,
): Promise<T> {
  if (isTauri()) {
    await ensureTauri();
    return _tauriInvoke!(cmd, args) as Promise<T>;
  }

  // 浏览器模式：HTTP fetch
  const isGet = cmd === "get_snapshot_status";
  const url = `${API_BASE}/api/${cmd}`;
  const options: RequestInit = {
    method: isGet ? "GET" : "POST",
    headers: { "Content-Type": "application/json" },
  };
  if (!isGet && args) {
    options.body = JSON.stringify(args);
  }
  const res = await fetch(url, options);
  if (!res.ok) {
    const text = await res.text().catch(() => "");
    throw new Error(`HTTP ${res.status}: ${text}`);
  }
  return res.json() as Promise<T>;
}

/**
 * 监听后端事件（浏览器模式下为空操作，canvas state 通过 HTTP 响应返回）。
 */
export async function listen<T = unknown>(
  _event: string,
  _handler: (event: { payload: T }) => void,
): Promise<() => void> {
  if (isTauri()) {
    await ensureTauri();
    return _tauriListen!(_event, _handler as (e: { payload: unknown }) => void);
  }
  // 浏览器模式下无事件系统，canvas state 在 HTTP 响应中直接返回。
  return () => {};
}
