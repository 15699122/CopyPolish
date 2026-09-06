import { useCallback, useEffect, useRef, useState } from "react";

import { formatText, normalizeCommandError, type CharacterConversion, type ReplacementPair, type RuleSelection } from "@/lib/tauri";

const NORMAL_DEBOUNCE_MS = 160;
const LONG_TEXT_DEBOUNCE_MS = 450;
const VERY_LONG_TEXT_DEBOUNCE_MS = 900;
const LONG_TEXT_THRESHOLD = 50_000;
const VERY_LONG_TEXT_THRESHOLD = 200_000;

export interface UseFormatterOptions {
  getSelection: (enabled: string[]) => RuleSelection;
}

export interface FormatOptions {
  replacements?: ReplacementPair[];
  conversion?: CharacterConversion;
}

export interface UseFormatterResult {
  output: string;
  error: string | null;
  isFormatting: boolean;
  lastFormatDuration: number | null;
  scheduleFormat: (
    input: string,
    enabled: string[],
    delayOverride?: number,
    options?: FormatOptions,
  ) => void;
  cancelFormat: () => void;
  clearOutput: () => void;
  clearError: () => void;
  reportError: (cause: unknown) => void;
}

function getFormatDebounceMs(text: string): number {
  if (text.length >= VERY_LONG_TEXT_THRESHOLD) return VERY_LONG_TEXT_DEBOUNCE_MS;
  if (text.length >= LONG_TEXT_THRESHOLD) return LONG_TEXT_DEBOUNCE_MS;
  return NORMAL_DEBOUNCE_MS;
}

export function useFormatter({ getSelection }: UseFormatterOptions): UseFormatterResult {
  const [output, setOutput] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [isFormatting, setIsFormatting] = useState(false);
  const [lastFormatDuration, setLastFormatDuration] = useState<number | null>(null);
  const debounceRef = useRef<number | null>(null);
  const sequenceRef = useRef(0);
  const callbacksRef = useRef({ getSelection });
  callbacksRef.current = { getSelection };

  const cancelFormat = useCallback(() => {
    if (debounceRef.current !== null) {
      window.clearTimeout(debounceRef.current);
      debounceRef.current = null;
    }
    sequenceRef.current += 1;
    setIsFormatting(false);
  }, []);

  const clearOutput = useCallback(() => setOutput(""), []);
  const clearError = useCallback(() => setError(null), []);
  const reportError = useCallback((cause: unknown) => setError(normalizeCommandError(cause).message), []);

  const scheduleFormat = useCallback(
    (nextInput: string, enabled: string[], delayOverride?: number, options: FormatOptions = {}) => {
      if (debounceRef.current !== null) window.clearTimeout(debounceRef.current);
      const delay = delayOverride ?? getFormatDebounceMs(nextInput);
      debounceRef.current = window.setTimeout(async () => {
        const sequence = ++sequenceRef.current;
        const startedAt = performance.now();
        setIsFormatting(true);
        try {
          const request = {
            text: nextInput,
            selection: callbacksRef.current.getSelection(enabled),
            ...(options.replacements !== undefined ? { replacements: options.replacements } : {}),
            ...(options.conversion !== undefined ? { conversion: options.conversion } : {}),
          };
          const result = await formatText(request);
          if (sequenceRef.current === sequence) {
            setOutput(result);
            setLastFormatDuration(Math.round(performance.now() - startedAt));
            setError(null);
          }
        } catch (cause) {
          if (sequenceRef.current === sequence) setError(normalizeCommandError(cause).message);
        } finally {
          if (sequenceRef.current === sequence) setIsFormatting(false);
        }
      }, delay);
    },
    [],
  );

  useEffect(() => {
    return () => {
      if (debounceRef.current !== null) window.clearTimeout(debounceRef.current);
      sequenceRef.current += 1;
    };
  }, []);

  return {
    output,
    error,
    isFormatting,
    lastFormatDuration,
    scheduleFormat,
    cancelFormat,
    clearOutput,
    clearError,
    reportError,
  };
}