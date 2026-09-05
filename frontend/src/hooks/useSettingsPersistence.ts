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

  const cancelScheduledPersist = useCallback(() => {
    if (timerRef.current === null) return;
    window.clearTimeout(timerRef.current);
    timerRef.current = null;
  }, []);

  const persistSettings = useCallback((patch: Partial<UserSettings> = {}) => {
    if (!callbacksRef.current.isHydrated()) return;
    // 立即保存必须取消旧的输入防抖保存，否则旧快照可能在本次设置后落盘。
    cancelScheduledPersist();
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
  }, [cancelScheduledPersist]);

  const schedulePersist = useCallback(
    (patch: Partial<UserSettings> = {}) => {
      cancelScheduledPersist();
      timerRef.current = window.setTimeout(() => {
        timerRef.current = null;
        persistSettings(patch);
      }, debounceMs);
    },
    [cancelScheduledPersist, debounceMs, persistSettings],
  );

  useEffect(() => {
    return () => {
      cancelScheduledPersist();
    };
  }, [cancelScheduledPersist]);

  return { settingsStatus, settingsError, persistSettings, schedulePersist };
}