import { useCallback, useRef } from "react";

interface UseVoiceRecognitionOptions {
  onResult: (text: string, isFinal: boolean) => void;
  onError: (error: string) => void;
  onEnd: () => void;
  lang?: string;
}

function isSpeechSupported(): boolean {
  return !!(window.SpeechRecognition || window.webkitSpeechRecognition);
}

export function useVoiceRecognition(options: UseVoiceRecognitionOptions) {
  const { onResult, onError, onEnd, lang = "zh-CN" } = options;
  const recognitionRef = useRef<SpeechRecognition | null>(null);
  // 防重入：连续快速点击时忽略
  const activeRef = useRef(false);

  const start = useCallback(() => {
    if (activeRef.current) return;
    activeRef.current = true;

    // 复用已有实例，避免重复创建 → 省去麦克风重新初始化延迟
    if (recognitionRef.current) {
      try {
        recognitionRef.current.start();
      } catch {
        // 实例可能已过期，重建
        recognitionRef.current = null;
      }
    }

    if (!recognitionRef.current) {
      const SpeechRecognition = window.SpeechRecognition || window.webkitSpeechRecognition;
      if (!SpeechRecognition) {
        activeRef.current = false;
        onError("浏览器不支持语音识别，请使用 Chrome 浏览器。");
        return;
      }

      const recognition = new SpeechRecognition();
      recognition.lang = lang;
      recognition.interimResults = true;
      recognition.continuous = true;
      recognition.maxAlternatives = 1;

      recognition.onresult = (event: SpeechRecognitionEvent) => {
        let transcript = "";
        let isFinal = false;
        for (let i = 0; i < event.results.length; i++) {
          transcript += event.results[i][0].transcript;
          if (event.results[i].isFinal) isFinal = true;
        }
        onResult(transcript, isFinal);
      };

      recognition.onerror = (event: SpeechRecognitionErrorEvent) => {
        if (event.error !== "no-speech" && event.error !== "aborted") {
          onError(`语音识别错误: ${event.error}`);
        }
      };

      recognition.onend = () => {
        activeRef.current = false;
        onEnd();
      };

      recognitionRef.current = recognition;
      recognition.start();
    }
  }, [lang, onResult, onError, onEnd]);

  const stop = useCallback(() => {
    activeRef.current = false;
    if (recognitionRef.current) {
      recognitionRef.current.stop();
    }
  }, []);

  const abort = useCallback(() => {
    activeRef.current = false;
    if (recognitionRef.current) {
      recognitionRef.current.abort();
      recognitionRef.current = null;
    }
  }, []);

  return { start, stop, abort, isSupported: isSpeechSupported() };
}
