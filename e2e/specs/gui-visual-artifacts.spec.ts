import {
  captureBrowserState,
  writeArtifactJson,
} from "../support/artifacts.js";
import { assertAppDidNotPolluteRepository, waitForApp } from "../support/app.js";

const enabled = process.env.COPYPOLISH_E2E_VISUAL_ARTIFACTS === "1";
const artifactDir = process.env.COPYPOLISH_E2E_ARTIFACT_DIR;
const expectedScale = Number(process.env.COPYPOLISH_E2E_EXPECTED_SCALE ?? "0");

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
    const display = await browser.execute(() => ({
      devicePixelRatio: window.devicePixelRatio,
      innerWidth: window.innerWidth,
      innerHeight: window.innerHeight,
      outerWidth: window.outerWidth,
      outerHeight: window.outerHeight,
      screenWidth: window.screen.width,
      screenHeight: window.screen.height,
      screenAvailableWidth: window.screen.availWidth,
      screenAvailableHeight: window.screen.availHeight,
      colorDepth: window.screen.colorDepth,
    }));
    const actualScale = Math.round(display.devicePixelRatio * 100);
    if (expectedScale > 0 && Math.abs(actualScale - expectedScale) > 1) {
      throw new Error(
        `真实 Windows DPI 不匹配：期望 ${expectedScale}%，WebView2 报告 ${actualScale}%`,
      );
    }
    await writeArtifactJson(artifactDir, "dpi-environment.json", {
      schemaVersion: 1,
      expectedScale: expectedScale || null,
      actualScale,
      display,
    });
    const recordState = async (name: string, theme: string, surface: string) => {
      const windowSize = await browser.getWindowSize();
      const metadata = { name, theme, surface, windowSize, actualScale, display };
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
      actualScale,
      display,
    });
  });
});