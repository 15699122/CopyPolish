import {
  assertAppDidNotPolluteRepository,
  formatText,
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
});