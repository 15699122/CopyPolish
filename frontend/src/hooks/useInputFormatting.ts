import { useCallback, useState } from "react";

import type { CharacterConversion, OutputMode, ReplacementPair, UserSettings } from "@/lib/tauri";

export interface UseInputFormattingOptions {
  enabled: string[];
  replacements?: ReplacementPair[];
  conversion?: CharacterConversion;
  outputMode?: OutputMode;
  scheduleFormat: (input: string, enabled: string[], delayOverride?: number, options?: {
    replacements?: ReplacementPair[];
    conversion?: CharacterConversion;
  }) => void;
  schedulePersist: (patch?: Partial<UserSettings>) => void;
}

export interface UseInputFormattingResult {
  input: string;
  setInput: (input: string) => void;
  onInputChange: (input: string) => void;
}

/** 管理输入值，以及输入变化触发的格式化和设置防抖保存。 */
export function useInputFormatting({
  enabled,
  replacements,
  conversion,
  outputMode = "realtime",
  scheduleFormat,
  schedulePersist,
}: UseInputFormattingOptions): UseInputFormattingResult {
  const [input, setInput] = useState("");

  const onInputChange = useCallback(
    (nextInput: string) => {
      setInput(nextInput);
      if (outputMode === "realtime") {
        scheduleFormat(nextInput, enabled, undefined, { replacements: replacements ?? [], conversion: conversion ?? "none" });
      }
      schedulePersist({ enabled, last_input: nextInput, replacements: replacements ?? [], conversion: conversion ?? "none" });
    },
    [conversion, enabled, outputMode, replacements, scheduleFormat, schedulePersist],
  );

  return { input, setInput, onInputChange };
}