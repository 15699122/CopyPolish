/**
 * App 组件交互测试（Vitest + Testing Library，jsdom）。
 * mock 掉 @/lib/tauri，专注 UI 行为：
 * 输入→实时排版、设置弹窗规则开关与持久化、清除输入、启动时恢复用户设置。
 */
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import App from "./App";

const mocks = vi.hoisted(() => {
  const rules = [
    { key: "rule-a", section: "空格", name: "中英文之间增加空格", disputed: false, default: true },
    { key: "rule-b", section: "空格", name: "中文与数字之间增加空格", disputed: false, default: true },
    { key: "rule-c", section: "争议", name: "争议规则", disputed: true, default: false },
  ];
  return {
    rules,
    formatText: vi.fn(),
    getRules: vi.fn(),
    getEnabledDefaults: vi.fn(),
    getUserSettings: vi.fn(),
    saveUserSettings: vi.fn(),
  };
});

vi.mock("@/lib/tauri", () => ({
  isTauri: () => false,
  formatText: mocks.formatText,
  getRules: mocks.getRules,
  getEnabledDefaults: mocks.getEnabledDefaults,
  getUserSettings: mocks.getUserSettings,
  saveUserSettings: mocks.saveUserSettings,
}));

// 默认排版实现：模拟引擎输出，便于断言防抖后的结果。
function mockFormat(transform: (text: string) => string) {
  mocks.formatText.mockImplementation((req: { text: string }) =>
    Promise.resolve(transform(req.text)),
  );
}

async function setup() {
  const user = userEvent.setup();
  render(<App />);
  // 等待初始化（getRules/getUserSettings）完成。
  await waitFor(() => expect(mocks.getRules).toHaveBeenCalled());
  return { user };
}

beforeEach(() => {
  vi.clearAllMocks();
  window.localStorage.clear();
  mocks.getRules.mockResolvedValue(mocks.rules);
  mocks.getEnabledDefaults.mockResolvedValue(["rule-a", "rule-b"]);
  mocks.getUserSettings.mockResolvedValue(null);
  mocks.saveUserSettings.mockResolvedValue(undefined);
});

describe("App 主流程", () => {
  it("输入后经防抖调用排版并展示输出", async () => {
    mockFormat((t) => `格式化(${t})`);
    const { user } = await setup();
    const input = screen.getByTestId("input-textarea");
    await user.type(input, "hello");
    await waitFor(
      () => expect(screen.getByTestId("output-text")).toHaveTextContent("格式化(hello)"),
      { timeout: 2000 },
    );
  });

  it("清除输入会清空输入框与输出区", async () => {
    mockFormat((t) => `格式化(${t})`);
    const { user } = await setup();
    const input = screen.getByTestId("input-textarea");
    await user.type(input, "abc");
    await waitFor(
      () => expect(screen.getByTestId("output-text")).toHaveTextContent("格式化(abc)"),
      { timeout: 2000 },
    );
    await user.click(screen.getByTestId("clear-input"));
    expect(input).toHaveValue("");
    await waitFor(() => expect(screen.getByTestId("output-text")).toBeEmptyDOMElement());
  });

  it("设置弹窗中开关规则会立即持久化用户设置", async () => {
    mockFormat((t) => t);
    const { user } = await setup();
    await user.click(screen.getByTestId("open-settings"));

    const checkboxC = screen.getByTestId("rule-rule-c");
    expect(checkboxC).not.toBeChecked();
    await user.click(checkboxC);
    expect(checkboxC).toBeChecked();

    await waitFor(() =>
      expect(mocks.saveUserSettings).toHaveBeenCalledWith({
        enabled: ["rule-a", "rule-b", "rule-c"],
        last_input: "",
      }),
    );

    // 关闭再打开，状态保持。
    await user.click(screen.getByTestId("settings-done"));
    await user.click(screen.getByTestId("open-settings"));
    expect(screen.getByTestId("rule-rule-c")).toBeChecked();
  });

  it("全不选会把启用集清空并重排", async () => {
    mockFormat((t) => `[${mocks.formatText.mock.calls.length}]${t}`);
    const { user } = await setup();
    await user.click(screen.getByTestId("open-settings"));
    await user.click(screen.getByTestId("select-none"));
    expect(screen.getByTestId("rule-rule-a")).not.toBeChecked();
    expect(screen.getByTestId("rule-rule-b")).not.toBeChecked();
    await waitFor(() =>
      expect(mocks.saveUserSettings).toHaveBeenCalledWith(
        expect.objectContaining({ enabled: [] }),
      ),
    );
  });

  it("启动时恢复上次保存的设置与输入内容", async () => {
    mocks.getUserSettings.mockResolvedValue({
      enabled: ["rule-c"],
      last_input: "上次输入",
    });
    mockFormat((t) => `格式化(${t})`);
    const { user } = await setup();
    expect(screen.getByTestId("input-textarea")).toHaveValue("上次输入");
    await waitFor(
      () => expect(screen.getByTestId("output-text")).toHaveTextContent("格式化(上次输入)"),
      { timeout: 2000 },
    );
    await user.click(screen.getByTestId("open-settings"));
    expect(screen.getByTestId("rule-rule-c")).toBeChecked();
    expect(screen.getByTestId("rule-rule-a")).not.toBeChecked();
  });
});
