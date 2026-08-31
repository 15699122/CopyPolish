import {
  assertAppDidNotPolluteRepository,
  formatText,
  waitForApp,
} from "../support/app.js";

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

  it("保存设置并在第二次启动恢复规则与最近输入", async () => {
    const source = "重启后应恢复的输入：在LeanCloud上，花了5000元";
    const input = await $("[data-testid=\"input-textarea\"]");

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
        const rules = await $$('[data-testid^="rule-"]');
        return (await rules.length) > 0;
      },
      { timeout: 10_000, timeoutMsg: "第二次启动规则列表未加载" },
    );
    const rules = await $$('[data-testid^="rule-"]');
    for (const rule of rules) {
      expect(await rule.getAttribute("data-state")).toBe("unchecked");
    }
    await $("[data-testid=\"settings-done\"]").click();

    const output = await formatText(source);
    expect(output).toBe(source);
  });
});