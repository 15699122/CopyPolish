import { assertAppDidNotPolluteRepository, formatText, waitForApp } from "../../support/app.js";
import { readSettings } from "../../support/settings.js";

describe("CopyPolish W3C provider 兼容性 smoke", () => {
  before(async () => {
    await waitForApp();
  });

  after(async () => {
    await assertAppDidNotPolluteRepository();
  });

  it("通过标准 W3C WebDriver session 发现主窗口并格式化默认示例", async () => {
    await expect(await formatText("在LeanCloud上，花了5000元")).toBe(
      "在 LeanCloud 上，花了 5000 元",
    );
  });

  it("标准 provider 下设置保存链路正常", async () => {
    await $("[data-testid=\"open-settings\"]").click();
    await browser.waitUntil(
      async () => (await $("[data-testid=\"select-none\"]")).isExisting(),
      { timeout: 10_000, timeoutMsg: "设置弹窗未完成渲染" },
    );
    await $("[data-testid=\"select-none\"]").click();
    await browser.waitUntil(
      async () => (await (await $("[data-testid=\"settings-status\"]")).getText()) === "设置已保存",
      { timeout: 10_000, timeoutMsg: "设置保存未完成" },
    );
    await $("[data-testid=\"settings-done\"]").click();

    const settingsDir = process.env.COPYPOLISH_E2E_SETTINGS_DIR;
    expect(settingsDir).toBeTruthy();
    const settings = await readSettings(settingsDir!);
    expect(settings).toContain("enabled: []");
  });
});
