import { useCallback, useEffect, useRef, useState } from "react";

import { saveUserSettings, type UserSettings } from "@/lib/tauri";

export type SettingsStatus = "idle" | "saving" | "saved" | "error";

export interface UseSettingsPersistenceOptions {
  getSettings: () => UserSettings;
  isHydrated: () => boolean;
  debounceMs: number;
}

export interface UseSettingsPersistenceResult {
  settingsStatus: SettingsStatus;
  settingsError: string | null;
  persistSettings: (patch?: Partial<UserSettings>) => void;
  schedulePersist: (patch?: Partial<UserSettings>) => void;
}

/** 设置保存生命周期：保存状态、错误、输入防抖和卸载时的定时器清理。 */
export function useSettingsPersistence({
  getSettings,
  isHydrated,
  debounceMs,
}: UseSettingsPersistenceOptions): UseSettingsPersistenceResult {
  const [settingsStatus, setSettingsStatus] = useState<SettingsStatus>("idle");
  const [settingsError, setSettingsError] = useState<string | null>(null);
  const timerRef = useRef<number | null>(null);
  const saveSequenceRef = useRef(0);
  const callbacksRef = useRef({ getSettings, isHydrated });
  callbacksRef.current = { getSettings, isHydrated };

  const persistSettings = useCallback((patch: Partial<UserSettings> = {}) => {
    if (!callbacksRef.current.isHydrated()) return;
    const sequence = ++saveSequenceRef.current;
    setSettingsStatus("saving");
    saveUserSettings({ ...callbacksRef.current.getSettings(), ...patch })
      .then(() => {
        if (saveSequenceRef.current !== sequence) return;
        setSettingsStatus("saved");
        setSettingsError(null);
      })
      .catch((cause) => {
        if (saveSequenceRef.current !== sequence) return;
        setSettingsStatus("error");
        setSettingsError(String(cause));
      });
  }, []);

  const schedulePersist = useCallback(
    (patch: Partial<UserSettings> = {}) => {
      if (timerRef.current !== null) window.clearTimeout(timerRef.current);
      timerRef.current = window.setTimeout(() => persistSettings(patch), debounceMs);
    },
    [debounceMs, persistSettings],
  );

  useEffect(() => {
    return () => {
      if (timerRef.current !== null) window.clearTimeout(timerRef.current);
    };
  }, []);

  return { settingsStatus, settingsError, persistSettings, schedulePersist };
}