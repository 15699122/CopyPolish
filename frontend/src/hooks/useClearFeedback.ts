import { useCallback, useEffect, useRef, useState } from "react";

export interface UseClearFeedbackOptions {
  clearInput: () => void;
  clearOutput: () => void;
  cancelFormat: () => void;
  clearError: () => void;
  persistEmptyInput: () => void;
  durationMs: number;
}

export interface UseClearFeedbackResult {
  cleared: boolean;
  clear: () => void;
}

/** 集中管理清空输入后的清理动作和短暂完成反馈，并负责定时器清理。 */
export function useClearFeedback({
  clearInput,
  clearOutput,
  cancelFormat,
  clearError,
  persistEmptyInput,
  durationMs,
}: UseClearFeedbackOptions): UseClearFeedbackResult {
  const [cleared, setCleared] = useState(false);
  const timerRef = useRef<number | null>(null);

  const clear = useCallback(() => {
    clearInput();
    clearOutput();
    cancelFormat();
    clearError();
    setCleared(true);
    if (timerRef.current !== null) window.clearTimeout(timerRef.current);
    timerRef.current = window.setTimeout(() => {
      timerRef.current = null;
      setCleared(false);
    }, durationMs);
    persistEmptyInput();
  }, [cancelFormat, clearError, clearInput, clearOutput, durationMs, persistEmptyInput]);

  useEffect(() => {
    return () => {
      if (timerRef.current !== null) window.clearTimeout(timerRef.current);
    };
  }, []);

  return { cleared, clear };
}