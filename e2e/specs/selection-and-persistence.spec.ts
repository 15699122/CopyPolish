import {
  assertAppDidNotPolluteRepository,
  formatText,
  getE2EDiagnostics,
  setTextInput,
  waitForApp,
} from "../support/app.js";
import { readSettings } from "../support/settings.js";

describe("CopyPolish 真实设置链路", () => {
  before(async () => {
    await waitForApp();
  });

  after(async () => {
    await assertAppDidNotPolluteRepository();
  });

  it("全不选时真实 IPC 输出保持恒等", async () => {
    await $("[data-testid=\"open-settings\"]").click();
    await browser.waitUntil(async () => await (await $("[data-testid=\"select-none\"]")).isExisting(), { timeout: 10_000, timeoutMsg: "设置弹窗未完成渲染" });
    await $("[data-testid=\"select-none\"]").click();
    await browser.waitUntil(async () => await (await $("[data-testid=\"settings-status\"]")).getText() === "设置已保存", { timeout: 10_000, timeoutMsg: "设置保存未完成" });
    await $("[data-testid=\"settings-done\"]").click();

    const source = "在LeanCloud上，花了5000元！！";
    await expect(await formatText(source)).toBe(source);
  });

  it("设置窗口暴露真实 rules.yaml 路径并保存规则选择", async () => {
    await $("[data-testid=\"open-settings\"]").click();
    const pathText = await $("[data-testid=\"settings-path\"]").getText();
    expect(pathText).toContain("rules.yaml");

    await browser.waitUntil(async () => await (await $("[data-testid=\"select-none\"]")).isExisting(), { timeout: 10_000, timeoutMsg: "设置弹窗未完成渲染" });
    await $("[data-testid=\"select-none\"]").click();
    await browser.waitUntil(async () => await (await $("[data-testid=\"settings-status\"]")).getText() === "设置已保存", { timeout: 10_000, timeoutMsg: "设置保存未完成" });
    await $("[data-testid=\"settings-done\"]").click();

    const settingsDir = process.env.COPYPOLISH_E2E_SETTINGS_DIR;
    expect(settingsDir).toBeTruthy();
    const settings = await readSettings(settingsDir!);
    expect(settings).toContain("enabled: []");
  });

  it("默认构建真实 GUI 保存替换项并拒绝不可用简繁转换", async () => {
    await $("[data-testid=\"open-settings\"]").click();
    const addReplacement = await $("[data-testid=\"replacement-add\"]");
    await addReplacement.waitForDisplayed({ timeout: 10_000 });
    await expect($("[data-testid=\"conversion-select\"]")).toHaveValue("none");
    await expect($("[data-testid=\"conversion-select\"] option[value=\"t2s\"]")).toBeDisabled();
    await expect($("[data-testid=\"conversion-select\"] option[value=\"s2t\"]")).toBeDisabled();
    await $("[data-testid=\"select-all\"]").click();
    await browser.waitUntil(
      async () => (await (await $("[data-testid=\"settings-status\"]")).getText()) === "设置已保存",
      { timeout: 10_000, timeoutMsg: "恢复默认规则选择保存未完成" },
    );
    await addReplacement.click();

    await $("[data-testid=\"replacement-from-0\"]").setValue("TODO");
    await browser.waitUntil(async () => await (await $("[data-testid=\"replacement-from-0\"]")).getValue() === "TODO", { timeout: 10_000, timeoutMsg: "替换来源未更新" });
    await $("[data-testid=\"replacement-to-0\"]").setValue("待办");
    await browser.waitUntil(async () => await (await $("[data-testid=\"replacement-to-0\"]")).getValue() === "待办", { timeout: 10_000, timeoutMsg: "替换目标未更新" });
    const saveBeforeConversion = Number((await getE2EDiagnostics()).settingsSaveSequence ?? 0);
    await browser.execute(() => {
      const select = document.querySelector<HTMLSelectElement>("[data-testid=\"conversion-select\"]");
      if (!select) throw new Error("简繁转换选择器不存在");
      const setter = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, "value")?.set;
      setter?.call(select, "s2t");
      select.dispatchEvent(new Event("change", { bubbles: true }));
    });
    await expect($("[data-testid=\"conversion-select\"]")).toHaveValue("none");

    await browser.waitUntil(
      async () => (await (await $("[data-testid=\"settings-status\"]")).getText()) === "设置已保存",
      { timeout: 10_000, timeoutMsg: "替换和转换设置保存未完成" },
    );
    await browser.waitUntil(
      async () => Number((await getE2EDiagnostics()).settingsSaveSequence ?? 0) > saveBeforeConversion,
      { timeout: 10_000, timeoutMsg: "本轮替换/转换设置保存序号未更新" },
    );

    const settingsDir = process.env.COPYPOLISH_E2E_SETTINGS_DIR;
    expect(settingsDir).toBeTruthy();
    const settings = await readSettings(settingsDir!);
    expect(settings).toContain("replacements:");
    expect(settings).toContain("from: TODO");
    expect(settings).toContain("to: 待办");
    expect(settings).toContain("conversion: none");

    await $("[data-testid=\"settings-done\"]").click();
    const output = await $("[data-testid=\"output-text\"]");
    await setTextInput("TODO");
    await browser.waitUntil(
      async () => (await output.getText()).includes("待办"),
      { timeout: 15_000, timeoutMsg: `替换设置未作用于真实 GUI 输出：${JSON.stringify(await getE2EDiagnostics())}` },
    );
    expect(await output.getText()).toContain("待办");
  });
});