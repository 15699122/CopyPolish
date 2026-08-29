import { useCallback, useState } from "react";

import type { UserSettings } from "@/lib/tauri";

export interface UseInputFormattingOptions {
  enabled: string[];
  scheduleFormat: (input: string, enabled: string[]) => void;
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
  scheduleFormat,
  schedulePersist,
}: UseInputFormattingOptions): UseInputFormattingResult {
  const [input, setInput] = useState("");

  const onInputChange = useCallback(
    (nextInput: string) => {
      setInput(nextInput);
      scheduleFormat(nextInput, enabled);
      schedulePersist({ enabled, last_input: nextInput });
    },
    [enabled, scheduleFormat, schedulePersist],
  );

  return { input, setInput, onInputChange };
}