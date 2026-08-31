import {
  assertAppDidNotPolluteRepository,
  formatText,
  waitForApp,
} from "../support/app.js";
import {
  expectedFixtureNotice,
  type SettingsFixture,
} from "../support/settings-fixtures.js";

const fixture = process.env.COPYPOLISH_E2E_SETTINGS_FIXTURE as SettingsFixture | undefined;

describe(`CopyPolish 损坏设置 fixture${fixture ? `：${fixture}` : ""}`, () => {
  if (!fixture) {
    it.skip("需要通过 test:corrupt-settings runner 提供 fixture", () => {});
    return;
  }

  before(async () => {
    await waitForApp();
  });

  after(async () => {
    await assertAppDidNotPolluteRepository();
  });

  it("显示恢复/降级提醒并继续通过真实 Rust IPC 排版", async () => {
    const notices = await $("[data-testid=\"settings-load-notices\"]");
    await browser.waitUntil(
      async () => (await notices.getText()).length > 0,
      { timeout: 10_000, timeoutMsg: "损坏设置提醒未显示" },
    );
    expect(await notices.getText()).toContain(expectedFixtureNotice(fixture));

    const source = "在LeanCloud上，花了5000元";
    const output = await formatText(source);
    if (fixture === "primary-corrupt-backup-valid") {
      expect(output).toBe("在 LeanCloud 上，花了 5000 元");
    } else {
      expect(output).toBe(source);
    }
  });
});