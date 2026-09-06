import { describe, expect, it } from "vitest";

import { normalizeCommandError } from "./tauri";

describe("normalizeCommandError", () => {
  it("保留受支持的错误 code，但使用固定安全消息", () => {
    const result = normalizeCommandError({
      code: "settings_path_unsafe",
      message: "原始路径 /home/user/private/rules.yaml",
    });

    expect(result).toEqual({
      code: "settings_path_unsafe",
      message: "设置路径不安全，已拒绝写入。",
    });
    expect(result.message).not.toContain("/home/user");
  });

  it("未知 code、Error 和字符串异常均回退为内部安全错误", () => {
    expect(normalizeCommandError({ code: "unknown", message: "secret" })).toEqual({
      code: "internal",
      message: "操作失败，请检查输入后重试。",
    });
    expect(normalizeCommandError(new Error("disk path /private/secret"))).toEqual({
      code: "internal",
      message: "操作失败，请检查输入后重试。",
    });
    expect(normalizeCommandError("token=secret")).toEqual({
      code: "internal",
      message: "操作失败，请检查输入后重试。",
    });
  });
});