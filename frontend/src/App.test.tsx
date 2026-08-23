/**
 * App 组件交互测试（Vitest + Testing Library，jsdom）。
 * mock 掉 @/lib/tauri，专注 UI 行为：
 * 输入→实时排版、设置弹窗规则开关与持久化、清除输入、启动时恢复用户设置。
 */
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import App from "./App";

// jsdom 不实现 window.matchMedia，需 mock 以支持主题 effect。
beforeEach(() => {
  Object.defineProperty(window, "matchMedia", {
    writable: true,
    value: vi.fn().mockImplementation((query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  });
});

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
    getAppVersion: vi.fn(),
  };
});

vi.mock("@/lib/tauri", () => ({
  isTauri: () => false,
  formatText: mocks.formatText,
  getRules: mocks.getRules,
  getEnabledDefaults: mocks.getEnabledDefaults,
  getSettingsPath: () => "C:\\Users\\Tester\\AppData\\Roaming\\CopyPolish\\settings.json",
  getUserSettings: mocks.getUserSettings,
  saveUserSettings: mocks.saveUserSettings,
  getAppVersion: mocks.getAppVersion,
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
  mocks.getAppVersion.mockResolvedValue("0.4.0");
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
    expect(screen.getByTestId("output-empty-state")).toBeInTheDocument();
  });

  it("输入框显示示例型占位符，输出框空状态显示引导提示", async () => {
    await setup();
    const input = screen.getByTestId("input-textarea");
    expect(input).toHaveAttribute(
      "placeholder",
      "请输入或粘贴中文文案，例如：在LeanCloud上，花了5000元",
    );
    expect(input).toHaveClass("placeholder:text-sm", "placeholder:text-muted-foreground/50");
    expect(screen.getByTestId("output-empty-state")).toHaveTextContent(
      "输入内容后，这里将实时显示规范化结果",
    );
  });

  it("输入内容后隐藏输出框空状态提示", async () => {
    mockFormat((t) => `格式化(${t})`);
    const { user } = await setup();
    await user.type(screen.getByTestId("input-textarea"), "hello");
    await waitFor(() =>
      expect(screen.getByTestId("output-text")).toHaveTextContent("格式化(hello)"),
    );
    expect(screen.queryByTestId("output-empty-state")).not.toBeInTheDocument();
  });

  it("设置弹窗使用稳定滚动布局并完整显示主题、规则和底部操作", async () => {
    const { user } = await setup();
    await user.click(screen.getByTestId("open-settings"));
    const dialog = screen.getByTestId("settings-dialog");
    expect(dialog).toBeVisible();
    expect(dialog).toHaveClass(
      "h-[min(680px,calc(100vh-2rem))]",
      "w-[min(560px,calc(100vw-2rem))]",
      "max-h-[calc(100vh-2rem)]",
      "max-w-[calc(100vw-2rem)]",
      "sm:min-h-[520px]",
      "sm:min-w-[480px]",
    );
    expect(screen.getByText("设置 — 排版规则")).toBeVisible();
    expect(screen.getByText("主题")).toBeVisible();
    expect(screen.getByTestId("settings-scroll-area")).toBeInTheDocument();
    expect(screen.queryByTestId("settings-drag-region")).not.toBeInTheDocument();
    expect(screen.queryByTestId("settings-resize-handle")).not.toBeInTheDocument();
    expect(screen.getByTestId("settings-footer")).toBeInTheDocument();
    expect(screen.getByTestId("settings-file-info")).toHaveTextContent("设置文件：");
    expect(screen.getByTestId("settings-version")).toHaveTextContent("版本 0.4.0");
    expect(screen.getByTestId("settings-footer")).toHaveClass("px-4", "py-4", "sm:px-6");
    expect(screen.getByTestId("settings-actions")).toBeInTheDocument();
    // 主题选项横向网格容器。
    expect(screen.getByTestId("theme-options")).toBeInTheDocument();
    expect(screen.getByTestId("theme-system")).toBeInTheDocument();
    expect(screen.getByTestId("theme-light")).toBeInTheDocument();
    expect(screen.getByTestId("theme-dark")).toBeInTheDocument();
    expect(screen.getByTestId("theme-options")).not.toHaveClass("border");
    expect(screen.getByTestId("font-settings")).toBeInTheDocument();
    expect(screen.getByTestId("font-select")).toHaveValue("system");
    expect(screen.getByTestId("reset-font")).toBeInTheDocument();
    expect(screen.getByText("中英文之间增加空格")).toBeVisible();
    expect(screen.getByText("中文与数字之间增加空格")).toBeVisible();
    expect(screen.getByText("争议规则")).toBeVisible();
    // 辅助按钮与完成按钮均存在。
    expect(screen.getByTestId("select-all")).toBeInTheDocument();
    expect(screen.getByTestId("select-none")).toBeInTheDocument();
    expect(screen.getByTestId("reset-defaults")).toBeInTheDocument();
    expect(screen.getByTestId("settings-done")).toBeInTheDocument();
    // 设置文件位于底部左侧区域，完成按钮仍位于右侧操作区内。
    const actionRow = screen.getByTestId("settings-actions").parentElement;
    expect(actionRow).not.toBeNull();
    expect(screen.getByTestId("settings-actions")).toContainElement(screen.getByTestId("settings-done"));
    expect(screen.getByTestId("settings-actions")).toHaveClass("flex-wrap", "items-center");
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
        theme: "system",
        font: "system",
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
      theme: "dark",
      font: "pingfang",
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
    // 主题被正确恢复到深色。
    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
    expect(screen.getByTestId("font-select")).toHaveValue("pingfang");
  });

  it("主题切换会立即应用并持久化", async () => {
    mockFormat((t) => t);
    const { user } = await setup();
    await user.click(screen.getByTestId("open-settings"));

    // 默认 theme=system；切换到 dark。
    await user.click(screen.getByTestId("theme-dark"));
    expect(document.documentElement).toHaveAttribute("data-theme", "dark");

    await waitFor(() =>
      expect(mocks.saveUserSettings).toHaveBeenCalledWith(
        expect.objectContaining({ theme: "dark" }),
      ),
    );

    // 切换到 light。
    await user.click(screen.getByTestId("theme-light"));
    expect(document.documentElement).toHaveAttribute("data-theme", "light");
    await waitFor(() =>
      expect(mocks.saveUserSettings).toHaveBeenCalledWith(
        expect.objectContaining({ theme: "light" }),
      ),
    );
  });

  it("字体切换与恢复默认会立即应用并持久化", async () => {
    const { user } = await setup();
    await user.click(screen.getByTestId("open-settings"));

    await user.selectOptions(screen.getByTestId("font-select"), "pingfang");
    expect(screen.getByTestId("font-select")).toHaveValue("pingfang");
    expect(document.documentElement.style.getPropertyValue("--app-font-family")).toContain("PingFang SC");
    await waitFor(() =>
      expect(mocks.saveUserSettings).toHaveBeenCalledWith(
        expect.objectContaining({ font: "pingfang" }),
      ),
    );

    await user.click(screen.getByTestId("reset-font"));
    expect(screen.getByTestId("font-select")).toHaveValue("system");
    expect(document.documentElement.style.getPropertyValue("--app-font-family")).toContain("system-ui");
    await waitFor(() =>
      expect(mocks.saveUserSettings).toHaveBeenCalledWith(
        expect.objectContaining({ font: "system" }),
      ),
    );
  });
});
