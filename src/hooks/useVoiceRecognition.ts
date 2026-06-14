import { useCallback, useEffect, useRef } from "react";

interface UseVoiceRecognitionOptions {
  onResult: (text: string, isFinal: boolean) => void;
  onError: (error: string) => void;
  onEnd: () => void;
  lang?: string;
}

function isSpeechSupported(): boolean {
  return !!(window.SpeechRecognition || window.webkitSpeechRecognition);
}

/**
 * 语音识别 hook。
 * 关键优化：recognition 实例挂载时即启动并持续运行，
 * "开始/停止"只切换结果捕获开关，消除每次 start() 的音频流初始化延迟。
 */
export function useVoiceRecognition(options: UseVoiceRecognitionOptions) {
  const { onResult, onError, onEnd, lang = "zh-CN" } = options;
  const recognitionRef = useRef<SpeechRecognition | null>(null);
  // true = 正在捕获结果（用户点了麦克风）
  const capturingRef = useRef(false);
  // 累积文本
  const transcriptRef = useRef("");

  // 挂载时启动 recognition，卸载时停止
  useEffect(() => {
    const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
    if (!SpeechRecognition) return;

    const recognition = new SpeechRecognition();
    recognition.lang = lang;
    recognition.interimResults = true;
    recognition.continuous = true;
    recognition.maxAlternatives = 1;

    recognition.onresult = (event: SpeechRecognitionEvent) => {
      if (!capturingRef.current) return;
      let t = "";
      let final = false;
      for (let i = 0; i < event.results.length; i++) {
        t += event.results[i][0].transcript;
        if (event.results[i].isFinal) final = true;
      }
      transcriptRef.current = t;
      onResult(t, final);
    };

    recognition.onerror = (event: SpeechRecognitionErrorEvent) => {
      if (event.error === "no-speech" || event.error === "aborted") return;
      onError(`语音识别错误: ${event.error}`);
    };

    recognition.onend = () => {
      // continuous=true 时，onend 通常在 abort 后触发
      // 如果不是主动 abort（capturing 为 true 时意外断开），尝试重启
      if (capturingRef.current) {
        try { recognition.start(); } catch { /* 忽略 */ }
      }
      onEnd();
    };

    recognition.start();
    recognitionRef.current = recognition;

    return () => {
      try { recognition.abort(); } catch { /* 忽略 */ }
    };
  }, [lang]); // 只挂载一次

  const start = useCallback(() => {
    transcriptRef.current = "";
    // 重启 recognition 清空浏览器语音缓冲区，防止上次录音残留
    const r = recognitionRef.current;
    if (r) {
      try { r.abort(); } catch { /* 忽略 */ }
      try { r.start(); } catch { /* 忽略 */ }
    }
    capturingRef.current = true;
  }, []);

  const stop = useCallback(() => {
    capturingRef.current = false;
  }, []);

  const abort = useCallback(() => {
    capturingRef.current = false;
    transcriptRef.current = "";
    const r = recognitionRef.current;
    if (r) {
      try { r.abort(); } catch { /* 忽略 */ }
      try { r.start(); } catch { /* 忽略 */ }
    }
  }, []);

  return { start, stop, abort, isSupported: isSpeechSupported() };
}
