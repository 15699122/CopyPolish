import { useCallback, useEffect, useRef, useState } from "react";

export interface UseClipboardStatusOptions {
  getText: () => string;
  onError: (cause: unknown) => void;
  resetMs: number;
}

export interface UseClipboardStatusResult {
  copied: boolean;
  copy: () => Promise<void>;
}

/** 剪贴板复制生命周期：写入文本、成功反馈、错误回调和自动复位。 */
export function useClipboardStatus({
  getText,
  onError,
  resetMs,
}: UseClipboardStatusOptions): UseClipboardStatusResult {
  const [copied, setCopied] = useState(false);
  const resetTimerRef = useRef<number | null>(null);
  const callbacksRef = useRef({ getText, onError });
  callbacksRef.current = { getText, onError };

  const copy = useCallback(async () => {
    const text = callbacksRef.current.getText();
    if (!text) return;

    try {
      await navigator.clipboard.writeText(text);
    } catch (cause) {
      callbacksRef.current.onError(cause);
      return;
    }

    if (resetTimerRef.current !== null) window.clearTimeout(resetTimerRef.current);
    setCopied(true);
    resetTimerRef.current = window.setTimeout(() => {
      resetTimerRef.current = null;
      setCopied(false);
    }, resetMs);
  }, [resetMs]);

  useEffect(() => {
    return () => {
      if (resetTimerRef.current !== null) window.clearTimeout(resetTimerRef.current);
    };
  }, []);

  return { copied, copy };
}