/**
 * App 组件交互测试（Vitest + Testing Library，jsdom）。
 * mock 掉 @/lib/tauri，专注 UI 行为：
 * 输入→实时排版、设置弹窗规则开关与持久化、清除输入、启动时恢复用户设置。
 */
import { render, screen, waitFor, fireEvent } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";

import App, { APP_NAME } from "./App";

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

const clipboardWriteText = vi.fn();

vi.mock("@/lib/tauri", () => ({
  isTauri: () => false,
  formatText: mocks.formatText,
  getRules: mocks.getRules,
  getEnabledDefaults: mocks.getEnabledDefaults,
  getSettingsPath: () => "C:\\Users\\Tester\\Desktop\\CopyPolish\\rules.yaml",
  getUserSettings: mocks.getUserSettings,
  saveUserSettings: mocks.saveUserSettings,
  getAppVersion: mocks.getAppVersion,
  DEFAULT_SHORTCUT_SETTINGS: {
    enabled: true,
    bindings: {
      format_now: "CtrlOrCmd+Enter",
      copy_output: "CtrlOrCmd+Shift+KeyC",
      open_settings: "CtrlOrCmd+Comma",
    },
  },
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
  Object.defineProperty(navigator, "clipboard", {
    configurable: true,
    value: { writeText: clipboardWriteText },
  });
  vi.clearAllMocks();
  clipboardWriteText.mockResolvedValue(undefined);
  window.localStorage.clear();
  mocks.getRules.mockResolvedValue(mocks.rules);
  mocks.getEnabledDefaults.mockResolvedValue(["rule-a", "rule-b"]);
  mocks.getUserSettings.mockResolvedValue(null);
  mocks.saveUserSettings.mockResolvedValue(undefined);
  mocks.getAppVersion.mockResolvedValue("0.5.0-test");
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
    expect(mocks.formatText).toHaveBeenLastCalledWith({
      text: "hello",
      selection: { mode: "only", keys: ["rule-a", "rule-b"] },
    });
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
      "请在这里粘贴或输入文字",
    );
    expect(input).toHaveClass("editor-text", "placeholder:text-muted-foreground/50");
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

  it("长文本显示处理提示并使用长文本排版流程", async () => {
    mockFormat((t) => t);
    const { user } = await setup();
    const longText = "中".repeat(50_000);
    await user.clear(screen.getByTestId("input-textarea"));
    await user.click(screen.getByTestId("input-textarea"));
    await user.paste(longText);

    expect(screen.getByTestId("long-text-status")).toHaveTextContent(
      "文本较长，处理可能需要更长时间",
    );
    await waitFor(
      () => expect(mocks.formatText).toHaveBeenCalledWith(expect.objectContaining({ text: longText })),
      { timeout: 2500 },
    );
  });

  it("键盘快捷键支持立即排版、复制和打开设置", async () => {
    mockFormat((t) => `格式化(${t})`);
    const { user } = await setup();
    const input = screen.getByTestId("input-textarea");
    await user.type(input, "hello");

    await user.keyboard("{Control>}{Enter}{/Control}");
    await waitFor(() => expect(mocks.formatText).toHaveBeenCalledWith(expect.objectContaining({ text: "hello" })));

    await waitFor(() => expect(screen.getByTestId("output-text")).toHaveTextContent("格式化(hello)"));
    await user.keyboard("{Control>}{Shift>}c{/Shift}{/Control}");
    await waitFor(() => expect(screen.getByTestId("copy-status")).toHaveTextContent("已复制"));

    document.body.focus();
    await user.keyboard("{Control>},{/Control}");
    expect(screen.getByTestId("settings-dialog")).toBeInTheDocument();
    await user.click(screen.getByTestId("settings-done"));
    await waitFor(() => expect(screen.getByTestId("open-settings")).toHaveFocus());
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
      "sm:min-h-130",
      "sm:min-w-120",
    );
        // 标题栏标题与说明之间保持明确间距。
    expect(screen.getByText(APP_NAME).parentElement).toHaveClass("space-y-1.5");
    expect(screen.getByText("设置 — 排版规则")).toBeVisible();
    expect(screen.getByText("主题")).toBeVisible();
    expect(screen.getByTestId("settings-scroll-area")).toBeInTheDocument();
    expect(screen.queryByTestId("settings-drag-region")).not.toBeInTheDocument();
    expect(screen.queryByTestId("settings-resize-handle")).not.toBeInTheDocument();
    expect(screen.getByTestId("settings-footer")).toBeInTheDocument();
    expect(screen.getByTestId("settings-file-info")).toHaveTextContent("设置文件：");
    const settingsPathEl = screen.getByTestId("settings-path");
    expect(settingsPathEl).toHaveAttribute(
      "title",
      "C:\\Users\\Tester\\Desktop\\CopyPolish\\rules.yaml",
    );
    expect(settingsPathEl).toHaveAttribute(
      "aria-label",
      "设置文件完整路径：C:\\Users\\Tester\\Desktop\\CopyPolish\\rules.yaml",
    );
    expect(settingsPathEl).toHaveClass("underline", "decoration-dotted", "underline-offset-4");
    // 下划线只作用于路径本身，“设置文件：”标签不带下划线。
    expect(screen.getByTestId("settings-path-label")).not.toHaveClass("underline");
    expect(settingsPathEl).toHaveTextContent("C:\\Users\\Tester\\Desktop\\CopyPolish\\rules.yaml");
    expect(screen.queryByRole("tooltip")).not.toBeInTheDocument();
    expect(screen.getByTestId("settings-version")).toHaveTextContent("版本 0.5.0-test");
    expect(screen.getByTestId("settings-footer")).toHaveClass("px-4", "py-4", "sm:px-6");
    expect(screen.getByTestId("settings-actions")).toBeInTheDocument();
    // 主题：跟随系统为勾选框，浅色/深色为单选项（默认勾选跟随时禁用）。
    expect(screen.getByTestId("theme-options")).toBeInTheDocument();
    expect(screen.getByTestId("theme-system")).toBeInTheDocument();
    expect(screen.getByTestId("theme-system")).toHaveAttribute("type", "checkbox");
    expect(screen.getByTestId("theme-system")).toBeChecked();
    expect(screen.getByTestId("theme-light")).toBeDisabled();
    expect(screen.getByTestId("theme-dark")).toBeDisabled();
    expect(screen.getByTestId("font-settings")).toBeInTheDocument();
    expect(screen.getByTestId("font-select")).toHaveValue("system");
    expect(screen.getByTestId("reset-font")).toBeInTheDocument();
    expect(screen.getByTestId("editor-font-size-select")).toHaveValue("normal");
    expect(screen.getByTestId("ui-scale-select")).toHaveValue("normal");
    expect(screen.getByText("中英文之间增加空格")).toBeVisible();
    expect(screen.getByText("中文与数字之间增加空格")).toBeVisible();
    expect(screen.getByText("争议规则")).toBeVisible();
    // 默认开启的规则展示在默认关闭的规则之前（仅展示顺序，不影响执行顺序）。
    const defaultRule = screen.getByTestId("rule-rule-a");
    const disputedRule = screen.getByTestId("rule-rule-c");
    expect(defaultRule.compareDocumentPosition(disputedRule) & Node.DOCUMENT_POSITION_FOLLOWING).toBeTruthy();
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
        editor_font_size: "normal",
        ui_scale: "normal",
        shortcuts: {
          enabled: true,
          bindings: {
            format_now: "CtrlOrCmd+Enter",
            copy_output: "CtrlOrCmd+Shift+KeyC",
            open_settings: "CtrlOrCmd+Comma",
          },
        },
      }),
    );

    // 关闭再打开，状态保持。
    await user.click(screen.getByTestId("settings-done"));
    await user.click(screen.getByTestId("open-settings"));
    expect(screen.getByTestId("rule-rule-c")).toBeChecked();
  });

  it("全不选会把启用集清空并重排", async () => {
    mockFormat((t) => t);
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
    await waitFor(() =>
      expect(mocks.formatText).toHaveBeenCalledWith({
        text: "",
        selection: { mode: "none" },
      }),
    );
  });

  it("启动时恢复上次保存的设置与输入内容", async () => {
    mocks.getUserSettings.mockResolvedValue({
      enabled: ["rule-c"],
      last_input: "上次输入",
      theme: "dark",
      font: "pingfang",
      editor_font_size: "large",
      ui_scale: "small",
      notices: [],
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
    expect(screen.getByTestId("editor-font-size-select")).toHaveValue("large");
    expect(screen.getByTestId("ui-scale-select")).toHaveValue("small");
  });

  it("主题切换会立即应用并持久化", async () => {
    mockFormat((t) => t);
    const { user } = await setup();
    await user.click(screen.getByTestId("open-settings"));

    // 默认勾选“跟随系统”，浅色/深色被禁用；先取消跟随时自动按系统偏好（mock 为浅色）切换。
    await user.click(screen.getByTestId("theme-system"));
    expect(screen.getByTestId("theme-system")).not.toBeChecked();
    expect(document.documentElement).toHaveAttribute("data-theme", "light");
    await waitFor(() =>
      expect(mocks.saveUserSettings).toHaveBeenCalledWith(
        expect.objectContaining({ theme: "light" }),
      ),
    );

    // 取消跟随后可切换到 dark。
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

  it("取消跟随系统时按系统深色偏好切换到深色，重新勾选恢复跟随", async () => {
    mockFormat((t) => t);
    // 系统偏好为深色。
    Object.defineProperty(window, "matchMedia", {
      writable: true,
      value: vi.fn().mockImplementation((query: string) => ({
        matches: query === "(prefers-color-scheme: dark)",
        media: query,
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
      })),
    });
    const { user } = await setup();
    await user.click(screen.getByTestId("open-settings"));

    await user.click(screen.getByTestId("theme-system"));
    expect(document.documentElement).toHaveAttribute("data-theme", "dark");
    await waitFor(() =>
      expect(mocks.saveUserSettings).toHaveBeenCalledWith(
        expect.objectContaining({ theme: "dark" }),
      ),
    );

    // 重新勾选跟随系统。
    await user.click(screen.getByTestId("theme-system"));
    expect(screen.getByTestId("theme-system")).toBeChecked();
    expect(screen.getByTestId("theme-light")).toBeDisabled();
    await waitFor(() =>
      expect(mocks.saveUserSettings).toHaveBeenCalledWith(
        expect.objectContaining({ theme: "system" }),
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

  it("字号和缩放下拉框切换会立即应用并持久化", async () => {
    const { user } = await setup();
    await user.click(screen.getByTestId("open-settings"));

    // 输入框与输出框共享同一字号入口。
    expect(screen.getByTestId("input-textarea")).toHaveClass("editor-text");
    expect(screen.getByTestId("output-text")).toHaveClass("editor-text");

    // 输入卡片与输出卡片使用一致的等高布局约束。
    const cards = screen.getByTestId("input-textarea").closest(".bg-card");
    const outputCard = screen.getByTestId("output-text").closest(".bg-card");
    expect(cards).not.toBeNull();
    expect(outputCard).not.toBeNull();
    for (const cls of ["h-full", "min-h-0", "min-w-0", "flex-col"]) {
      expect(cards).toHaveClass(cls);
      expect(outputCard).toHaveClass(cls);
    }

    // 输出滚动容器与输入 textarea 的内容内边距保持一致（px-3 py-2）。
    const outputScroller = screen.getByTestId("output-text").parentElement!;
    expect(outputScroller).toHaveClass("px-3", "py-2");

    await user.selectOptions(screen.getByTestId("editor-font-size-select"), "large");
    expect(document.documentElement.style.getPropertyValue("--editor-font-size")).toBe("16px");
    await waitFor(() =>
      expect(mocks.saveUserSettings).toHaveBeenCalledWith(
        expect.objectContaining({ editor_font_size: "large" }),
      ),
    );

    await user.selectOptions(screen.getByTestId("ui-scale-select"), "small");
    expect(document.documentElement.style.getPropertyValue("--app-ui-scale")).toBe("0.9");
    await waitFor(() =>
      expect(mocks.saveUserSettings).toHaveBeenCalledWith(
        expect.objectContaining({ ui_scale: "small" }),
      ),
    );
  });

  it("设置加载提醒会显示在主界面和设置窗口", async () => {
    mocks.getUserSettings.mockResolvedValue({
      enabled: ["rule-a", "rule-b"],
      last_input: "",
      theme: "system",
      font: "system",
      editor_font_size: "normal",
      ui_scale: "normal",
      notices: ["primary_settings_corrupt_recovered_from_backup"],
    });
    const { user } = await setup();
    expect(screen.getByTestId("settings-load-notices")).toHaveTextContent(
      "设置文件损坏，已从 rules.yaml.bak 恢复。",
    );
    await user.click(screen.getByTestId("open-settings"));
    expect(screen.getByTestId("settings-load-notice-primary_settings_corrupt_recovered_from_backup")).toBeInTheDocument();
  });
});

describe("快捷键配置", () => {
  it("默认启用时组合键生效，关闭总开关后全部失效并持久化", async () => {
    mockFormat((t) => `格式化(${t})`);
    const { user } = await setup();
    const input = screen.getByTestId("input-textarea");
    await user.type(input, "hello");

    // 默认启用：Ctrl+Enter 触发排版。
    await user.keyboard("{Control>}{Enter}{/Control}");
    await waitFor(() =>
      expect(mocks.formatText).toHaveBeenCalledWith(
        expect.objectContaining({ text: "hello" }),
      ),
    );

    // 打开设置并关闭总开关。
    await user.click(screen.getByTestId("open-settings"));
    const toggle = screen.getByTestId("shortcuts-toggle");
    expect(toggle).toBeChecked();
    await user.click(toggle);
    expect(toggle).not.toBeChecked();
    await waitFor(() =>
      expect(mocks.saveUserSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          shortcuts: expect.objectContaining({ enabled: false }),
        }),
      ),
    );

    // 关闭后：先关掉设置弹窗，再验证组合键全部失效。
    await user.click(screen.getByTestId("settings-done"));
    await waitFor(() => expect(screen.getByTestId("open-settings")).toHaveFocus());
    mocks.formatText.mockClear();
    clipboardWriteText.mockClear();
    await user.keyboard("{Control>}{Enter}{/Control}");
    fireEvent.keyDown(input, { code: "KeyC", ctrlKey: true, shiftKey: true, key: "C" });
    fireEvent.keyDown(document.body, { code: "Comma", ctrlKey: true, key: "," });
    expect(mocks.formatText).not.toHaveBeenCalled();
    expect(clipboardWriteText).not.toHaveBeenCalled();
    expect(screen.queryByTestId("settings-dialog")).toBeNull();

    // 重新开启后恢复监听。
    await user.click(toggle);
    await waitFor(() =>
      expect(mocks.saveUserSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          shortcuts: expect.objectContaining({ enabled: true }),
        }),
      ),
    );
  });

  it("IME 组合态按键不触发快捷键", async () => {
    mockFormat((t) => `格式化(${t})`);
    const { user } = await setup();
    const input = screen.getByTestId("input-textarea");
    await user.type(input, "n");

    fireEvent.keyDown(input, {
      code: "Enter",
      ctrlKey: true,
      key: "Enter",
      isComposing: true,
      keyCode: 229,
    });
    expect(mocks.formatText).not.toHaveBeenCalled();
    expect(input).toHaveValue("n");
  });

  it("修改绑定后新组合键生效；重复绑定被拒绝", async () => {
    mockFormat((t) => `格式化(${t})`);
    const { user } = await setup();
    const input = screen.getByTestId("input-textarea");
    await user.type(input, "abc");

    await user.click(screen.getByTestId("open-settings"));
    const editButton = screen.getByTestId("shortcut-edit-format_now");
    expect(screen.getByTestId("shortcut-value-format_now")).toHaveTextContent("Ctrl/Cmd + Enter");
    await user.click(editButton);
    fireEvent.keyDown(editButton, { code: "KeyR", ctrlKey: true, key: "r" });
    await waitFor(() =>
      expect(mocks.saveUserSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          shortcuts: expect.objectContaining({
            bindings: expect.objectContaining({ format_now: "CtrlOrCmd+KeyR" }),
          }),
        }),
      ),
    );
    await user.click(screen.getByTestId("settings-done"));
    await waitFor(() => expect(screen.getByTestId("open-settings")).toHaveFocus());

    // 新绑定生效，旧绑定失效。
    // 清除输入阶段正常防抖排版产生的调用，只观察快捷键事件本身。
    mocks.formatText.mockClear();
    fireEvent.keyDown(input, { code: "Enter", ctrlKey: true, key: "Enter" });
    expect(mocks.formatText).not.toHaveBeenCalled();
    fireEvent.keyDown(input, { code: "KeyR", ctrlKey: true, key: "r" });
    await waitFor(() =>
      expect(mocks.formatText).toHaveBeenCalledWith(
        expect.objectContaining({ text: "abc" }),
      ),
    );

    // 重复绑定被拒绝：copy_output 尝试占用 Ctrl/Cmd+R。
    await user.click(screen.getByTestId("open-settings"));
    const copyEdit = screen.getByTestId("shortcut-edit-copy_output");
    await user.click(copyEdit);
    fireEvent.keyDown(copyEdit, { code: "KeyR", ctrlKey: true, key: "r" });
    expect(screen.getByTestId("shortcut-status")).toHaveTextContent("已被「立即排版」占用");
    expect(screen.getByTestId("shortcut-value-copy_output")).toHaveTextContent("Ctrl/Cmd + Shift + C");
  });

  it("恢复默认快捷键会持久化默认值并重启恢复关闭的总开关", async () => {
    mockFormat((t) => t);
    mocks.getUserSettings.mockResolvedValue({
      enabled: [],
      last_input: "",
      theme: "system",
      font: "system",
      editor_font_size: "normal",
      ui_scale: "normal",
      notices: [],
      shortcuts: {
        enabled: true,
        bindings: {
          format_now: "CtrlOrCmd+KeyR",
          copy_output: "CtrlOrCmd+Shift+KeyC",
          open_settings: "CtrlOrCmd+Comma",
        },
      },
    });
    const { user } = await setup();
    await user.click(screen.getByTestId("open-settings"));
    await user.click(screen.getByTestId("reset-shortcuts"));
    await waitFor(() =>
      expect(mocks.saveUserSettings).toHaveBeenCalledWith(
        expect.objectContaining({
          shortcuts: expect.objectContaining({
            enabled: true,
            bindings: expect.objectContaining({ format_now: "CtrlOrCmd+Enter" }),
          }),
        }),
      ),
    );
    expect(screen.getByTestId("shortcut-value-format_now")).toHaveTextContent("Ctrl/Cmd + Enter");
  });
});
