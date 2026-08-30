import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

const mocks = vi.hoisted(() => ({
  getRules: vi.fn(),
  getEnabledDefaults: vi.fn(),
}));

vi.mock("@/lib/tauri", () => ({
  getRules: mocks.getRules,
  getEnabledDefaults: mocks.getEnabledDefaults,
}));

import { useRuleCatalog } from "./useRuleCatalog";

const rules = [
  { key: "rule-a", section: "空格", name: "规则 A", disputed: false, default: true },
  { key: "rule-b", section: "空格", name: "规则 B", disputed: false, default: true },
];

describe("useRuleCatalog", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mocks.getRules.mockResolvedValue(rules);
    mocks.getEnabledDefaults.mockResolvedValue(["rule-a", "rule-b"]);
  });

  it("加载规则并触发设置恢复", async () => {
    const loadSettings = vi.fn().mockResolvedValue(undefined);
    const onError = vi.fn();
    const { result } = renderHook(() => useRuleCatalog({ loadSettings, onError }));

    await waitFor(() => expect(result.current.rules).toEqual(rules));
    expect(mocks.getRules).toHaveBeenCalledOnce();
    expect(mocks.getEnabledDefaults).toHaveBeenCalledOnce();
    expect(loadSettings).toHaveBeenCalledWith(rules, ["rule-a", "rule-b"]);
    expect(onError).not.toHaveBeenCalled();
  });

  it("加载失败时上报错误且不触发设置恢复", async () => {
    mocks.getRules.mockRejectedValue(new Error("registry empty"));
    const loadSettings = vi.fn();
    const onError = vi.fn();
    const { result } = renderHook(() => useRuleCatalog({ loadSettings, onError }));

    await act(async () => {
      await Promise.resolve();
    });

    expect(result.current.rules).toEqual([]);
    expect(loadSettings).not.toHaveBeenCalled();
    expect(onError).toHaveBeenCalledWith(expect.any(Error));
  });
});