import {
  captureBrowserState,
  writeArtifactJson,
} from "../support/artifacts.js";
import { assertAppDidNotPolluteRepository, waitForApp } from "../support/app.js";

const enabled = process.env.COPYPOLISH_E2E_VISUAL_ARTIFACTS === "1";
const artifactDir = process.env.COPYPOLISH_E2E_ARTIFACT_DIR;

describe("CopyPolish GUI 视觉 artifact", () => {
  if (!enabled || !artifactDir) {
    it.skip("需要通过 test:gui-visual-artifacts runner 显式启用", () => {});
    return;
  }

  before(async () => {
    await waitForApp();
  });

  after(async () => {
    await assertAppDidNotPolluteRepository();
  });

  it("采集主题、设置窗口和窄窗口状态", async () => {
    const states: Array<Record<string, unknown>> = [];
    const recordState = async (name: string, theme: string, surface: string) => {
      const windowSize = await browser.getWindowSize();
      const metadata = { name, theme, surface, windowSize };
      states.push(metadata);
      await captureBrowserState(artifactDir, name, metadata);
    };

    await recordState("main-normal", "initial", "main");

    await $("[data-testid=\"open-settings\"]").click();
    const systemTheme = await $("[data-testid=\"theme-system\"]");
    if (await systemTheme.getAttribute("data-state") === "checked") {
      await systemTheme.click();
      await browser.waitUntil(
        async () => await systemTheme.getAttribute("data-state") === "unchecked",
        { timeout: 10_000, timeoutMsg: "未能关闭跟随系统主题" },
      );
    }
    await $("[data-testid=\"theme-light\"]").click();
    await browser.waitUntil(
      async () => (await $("[data-testid=\"theme-light\"]")).isSelected(),
      { timeout: 10_000, timeoutMsg: "浅色主题未生效" },
    );
    await recordState("settings-light", "light", "settings");

    await $("[data-testid=\"theme-dark\"]").click();
    await browser.waitUntil(
      async () => (await $("[data-testid=\"theme-dark\"]")).isSelected(),
      { timeout: 10_000, timeoutMsg: "深色主题未生效" },
    );
    await recordState("settings-dark", "dark", "settings");

    await $("[data-testid=\"settings-done\"]").click();
    const originalSize = await browser.getWindowSize();
    await browser.setWindowSize(420, 700);
    await recordState("main-narrow", "dark", "main");

    await $("[data-testid=\"open-settings\"]").click();
    await recordState("settings-narrow", "dark", "settings");
    await $("[data-testid=\"settings-done\"]").click();
    await browser.setWindowSize(originalSize.width, originalSize.height);

    await writeArtifactJson(artifactDir, "visual-states.json", {
      schemaVersion: 1,
      states,
      restoredWindowSize: originalSize,
    });
  });
});