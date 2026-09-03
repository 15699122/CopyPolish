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

  it("立即保存会取消旧的防抖保存，避免旧快照覆盖最新设置", async () => {
    vi.useFakeTimers();
    try {
      const { result } = renderHook(() =>
        useSettingsPersistence({
          getSettings: () => settings,
          isHydrated: () => true,
          debounceMs: 800,
        }),
      );

      act(() => {
        result.current.schedulePersist({ conversion: "s2t", last_input: "旧输入" });
        result.current.persistSettings({ conversion: "t2s", last_input: "新设置" });
      });

      expect(mocks.saveUserSettings).toHaveBeenCalledOnce();
      expect(mocks.saveUserSettings).toHaveBeenCalledWith({
        ...settings,
        conversion: "t2s",
        last_input: "新设置",
      });

      act(() => {
        vi.advanceTimersByTime(800);
      });
      await act(async () => {
        await Promise.resolve();
      });
      expect(mocks.saveUserSettings).toHaveBeenCalledOnce();
    } finally {
      vi.useRealTimers();
    }
  });

  it("乱序完成时只允许最新保存更新状态", async () => {
    let resolveFirst!: () => void;
    let rejectSecond!: (cause: Error) => void;
    const firstSave = new Promise<void>((resolve) => { resolveFirst = resolve; });
    const secondSave = new Promise<void>((_, reject) => { rejectSecond = reject; });
    mocks.saveUserSettings
      .mockImplementationOnce(() => firstSave)
      .mockImplementationOnce(() => secondSave);

    const { result } = renderHook(() =>
      useSettingsPersistence({
        getSettings: () => settings,
        isHydrated: () => true,
        debounceMs: 20,
      }),
    );

    act(() => {
      result.current.persistSettings({ last_input: "first" });
      result.current.persistSettings({ last_input: "second" });
    });
    expect(result.current.settingsStatus).toBe("saving");

    await act(async () => {
      rejectSecond(new Error("latest save failed"));
    });
    await waitFor(() => expect(result.current.settingsStatus).toBe("error"));
    expect(result.current.settingsError).toContain("latest save failed");

    await act(async () => {
      resolveFirst();
    });
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(result.current.settingsStatus).toBe("error");
    expect(result.current.settingsError).toContain("latest save failed");
  });
});