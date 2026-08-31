import {
  assertAppDidNotPolluteRepository,
  formatText,
  waitForApp,
} from "../support/app.js";

const fixturePath = process.env.COPYPOLISH_E2E_ACL_SETTINGS_PATH;

describe("CopyPolish Windows NTFS ACL 设置保存失败", () => {
  if (process.platform !== "win32" || !fixturePath) {
    it.skip("需要 Windows 原生 runner 提供 NTFS ACL fixture", () => {});
    return;
  }

  before(async () => {
    await waitForApp();
  });

  after(async () => {
    await assertAppDidNotPolluteRepository();
  });

  it("显示带 rules.yaml 路径的保存失败提示且真实 IPC 仍可用", async () => {
    await $("[data-testid=\"open-settings\"]").click();
    const settingsPath = await $("[data-testid=\"settings-path\"]");
    expect(await settingsPath.getAttribute("title")).toBe(fixturePath);

    const selectNone = await $("[data-testid=\"select-none\"]");
    await selectNone.waitForDisplayed({ timeout: 10_000 });
    await selectNone.click();

    await browser.waitUntil(
      async () => {
        const status = await $("[data-testid=\"settings-status\"]");
        return (await status.getText()).includes("设置保存失败");
      },
      { timeout: 10_000, timeoutMsg: "NTFS ACL 拒写后未显示设置保存失败" },
    );
    const status = await $("[data-testid=\"settings-status\"]");
    expect(await status.getText()).toContain("rules.yaml");

    await $("[data-testid=\"settings-done\"]").click();
    const source = "在LeanCloud上，花了5000元";
    const output = await formatText(source);
    expect(output).toBe(source);
  });
});