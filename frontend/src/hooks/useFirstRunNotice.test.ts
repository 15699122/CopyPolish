import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";

import {
  FIRST_RUN_NOTICE_STORAGE_KEY,
  useFirstRunNotice,
} from "./useFirstRunNotice";

describe("useFirstRunNotice", () => {
  beforeEach(() => window.localStorage.clear());

  it("首次使用显示提示，关闭后持久化已查看状态", () => {
    const { result } = renderHook(() => useFirstRunNotice());

    expect(result.current.visible).toBe(true);
    act(() => result.current.dismiss());
    expect(result.current.visible).toBe(false);
    expect(window.localStorage.getItem(FIRST_RUN_NOTICE_STORAGE_KEY)).toBe("1");
  });

  it("已经查看过时不再显示提示", () => {
    window.localStorage.setItem(FIRST_RUN_NOTICE_STORAGE_KEY, "1");

    const { result } = renderHook(() => useFirstRunNotice());

    expect(result.current.visible).toBe(false);
  });
});