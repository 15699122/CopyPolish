import {
  assertAppDidNotPolluteRepository,
  formatText,
  getE2EDiagnostics,
  waitForApp,
} from "../support/app.js";
import { readSettings } from "../support/settings.js";

const phase = process.env.COPYPOLISH_E2E_RESTART_PHASE;

describe(`CopyPolish 设置重启恢复：${phase}`, () => {
  if (phase !== "write" && phase !== "read") {
    it.skip("需要通过 test:restart-settings runner 提供 write/read phase", () => {});
    return;
  }

  before(async () => {
    await waitForApp();
  });

  after(async () => {
    await assertAppDidNotPolluteRepository();
  });

  it("保存设置并在第二次启动恢复规则、替换、转换与最近输入", async () => {
    const source = "重启后应恢复的输入：在LeanCloud上，花了5000元";
    const input = await $("[data-testid=\"input-textarea\"]");
    const diagnostics = await getE2EDiagnostics();
    const simplifiedTradConversion = Boolean(
      (diagnostics.buildCapabilities as { simplifiedTradConversion?: unknown } | undefined)
        ?.simplifiedTradConversion,
    );

    if (phase === "write") {
      await input.setValue(source);
      await formatText(source);

      await $("[data-testid=\"open-settings\"]").click();
      const selectNone = await $("[data-testid=\"select-none\"]");
      await selectNone.waitForDisplayed({ timeout: 10_000 });
      await selectNone.click();
      await browser.waitUntil(
        async () => {
          const status = await $("[data-testid=\"settings-status\"]");
          return (await status.getText()) === "设置已保存";
        },
        { timeout: 10_000, timeoutMsg: "第一次启动的设置保存未完成" },
      );

      await $("[data-testid=\"replacement-add\"]").click();
      await $("[data-testid=\"replacement-from-0\"]").setValue("LeanCloud");
      await $("[data-testid=\"replacement-to-0\"]").setValue("LeanCloud服务");
      if (simplifiedTradConversion) {
        await browser.execute(() => {
          const select = document.querySelector<HTMLSelectElement>("[data-testid=\"conversion-select\"]");
          if (!select) throw new Error("第一次启动的简繁转换选择器不存在");
          const setter = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, "value")?.set;
          setter?.call(select, "t2s");
          select.dispatchEvent(new Event("change", { bubbles: true }));
        });
        await browser.waitUntil(
          async () => await (await $("[data-testid=\"conversion-select\"]")).getValue() === "t2s",
          { timeout: 10_000, timeoutMsg: "第一次启动的简繁转换选择未更新" },
        );
      } else {
        await expect($("[data-testid=\"conversion-select\"]")).toHaveValue("none");
        await expect($("[data-testid=\"conversion-select\"] option[value=\"t2s\"]")).toBeDisabled();
        await expect($("[data-testid=\"conversion-select\"] option[value=\"s2t\"]")).toBeDisabled();
      }
      await browser.waitUntil(
        async () => (await (await $("[data-testid=\"settings-status\"]")).getText()) === "设置已保存",
        { timeout: 10_000, timeoutMsg: "第一次启动的替换和转换设置保存未完成" },
      );
      const settingsDir = process.env.COPYPOLISH_E2E_SETTINGS_DIR;
      expect(settingsDir).toBeTruthy();
      const settings = await readSettings(settingsDir!);
      expect(settings).toContain(
        `conversion: ${simplifiedTradConversion ? "t2s" : "none"}`,
      );
      return;
    }

    await input.waitForDisplayed({ timeout: 10_000 });
    await browser.waitUntil(
      async () => await input.getValue() === source,
      { timeout: 10_000, timeoutMsg: "第二次启动未恢复最近输入" },
    );

    await $("[data-testid=\"open-settings\"]").click();
    await browser.waitUntil(
      async () => {
        const rules = await $$('[data-testid^="rule-"]:not([data-testid^="rule-card-"])');
        return (await rules.length) > 0;
      },
      { timeout: 10_000, timeoutMsg: "第二次启动规则列表未加载" },
    );
    const rules = await $$('[data-testid^="rule-"]:not([data-testid^="rule-card-"])');
    for (const rule of rules) {
      expect(await rule.getAttribute("data-state")).toBe("unchecked");
    }
    expect(await $("[data-testid=\"replacement-from-0\"]").getValue()).toBe("LeanCloud");
    expect(await $("[data-testid=\"replacement-to-0\"]").getValue()).toBe("LeanCloud服务");
    expect(await $("[data-testid=\"replacement-active-0\"]").getAttribute("data-state")).toBe("checked");
    expect(await $("[data-testid=\"conversion-select\"]").getValue()).toBe(
      simplifiedTradConversion ? "t2s" : "none",
    );
    const settingsDir = process.env.COPYPOLISH_E2E_SETTINGS_DIR;
    expect(settingsDir).toBeTruthy();
    const settings = await readSettings(settingsDir!);
    expect(settings).toContain(
      `conversion: ${simplifiedTradConversion ? "t2s" : "none"}`,
    );
    await $("[data-testid=\"settings-done\"]").click();

    const output = await formatText(source);
    expect(output).toContain("LeanCloud服务");
  });
});