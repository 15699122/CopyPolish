import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { useSettingsPersistence } from "./useSettingsPersistence";

const mocks = vi.hoisted(() => ({ saveUserSettings: vi.fn() }));

vi.mock("@/lib/tauri", () => ({ saveUserSettings: mocks.saveUserSettings }));

const settings = {
  enabled: ["rule-a"],
  last_input: "hello",
  theme: "system" as const,
  font: "system" as const,
  editor_font_size: "normal" as const,
  ui_scale: "normal" as const,
  shortcuts: {
    enabled: true,
    bindings: {
      format_now: "CtrlOrCmd+Enter",
      copy_output: "CtrlOrCmd+Shift+KeyC",
      open_settings: "CtrlOrCmd+Comma",
    },
  },
  replacements: [],
  conversion: "none" as const,
};

describe("useSettingsPersistence", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.saveUserSettings.mockResolvedValue(undefined);
  });

  it("未完成初始化时不保存，hydrate 后立即保存 patch", async () => {
    let hydrated = false;
    const { result } = renderHook(() =>
      useSettingsPersistence({
        getSettings: () => settings,
        isHydrated: () => hydrated,
        debounceMs: 20,
      }),
    );

    act(() => result.current.persistSettings({ last_input: "before" }));
    expect(mocks.saveUserSettings).not.toHaveBeenCalled();

    hydrated = true;
    act(() => result.current.persistSettings({ last_input: "after" }));
    await waitFor(() => expect(mocks.saveUserSettings).toHaveBeenCalledOnce());
    expect(mocks.saveUserSettings).toHaveBeenCalledWith({ ...settings, last_input: "after" });
    await waitFor(() => expect(result.current.settingsStatus).toBe("saved"));
  });

  it("防抖保存只执行最后一次 patch，并暴露保存错误", async () => {
    const { result } = renderHook(() =>
      useSettingsPersistence({
        getSettings: () => settings,
        isHydrated: () => true,
        debounceMs: 20,
      }),
    );

    act(() => {
      result.current.schedulePersist({ last_input: "first" });
      result.current.schedulePersist({ last_input: "second" });
    });
    await waitFor(() => expect(mocks.saveUserSettings).toHaveBeenCalledOnce());
    expect(mocks.saveUserSettings).toHaveBeenCalledWith({ ...settings, last_input: "second" });

    mocks.saveUserSettings.mockRejectedValueOnce(new Error("disk full"));
    act(() => result.current.persistSettings());
    await waitFor(() => expect(result.current.settingsStatus).toBe("error"));
    expect(result.current.settingsError).toContain("disk full");
  });
});