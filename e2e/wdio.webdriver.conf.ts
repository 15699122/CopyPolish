import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { startWebDriverApp, stopWebDriverApp, type WebDriverApp } from "./support/webdriver-app.js";
import { prepareSettingsFixture } from "./support/settings-fixtures.js";

const e2eDir = path.dirname(fileURLToPath(import.meta.url));
const artifactsDir = process.env.COPYPOLISH_E2E_ARTIFACT_DIR
  ?? path.join(e2eDir, "artifacts", "webdriver");
const port = Number(process.env.TAURI_WEBDRIVER_PORT ?? 4445);
let app: WebDriverApp | undefined;

prepareSettingsFixture(
  process.env.COPYPOLISH_E2E_SETTINGS_DIR ?? "",
  process.env.COPYPOLISH_E2E_SETTINGS_FIXTURE as Parameters<typeof prepareSettingsFixture>[1],
);

export const config: WebdriverIO.Config = {
  runner: "local",
  specs: ["./specs/**/*.spec.ts"],
  maxInstances: 1,
  hostname: "127.0.0.1",
  port,
  path: "/",
  capabilities: [{ browserName: "chrome", "goog:chromeOptions": {} }],
  framework: "mocha",
  reporters: ["spec"],
  logLevel: "info",
  outputDir: artifactsDir,
  waitforTimeout: 10_000,
  connectionRetryTimeout: 30_000,
  connectionRetryCount: 3,
  mochaOpts: { timeout: 60_000 },
  onPrepare: async () => {
    app = await startWebDriverApp(port);
  },
  afterTest: async (_test, _context, result) => {
    if (!result.passed) {
      await fs.mkdir(artifactsDir, { recursive: true });
      try {
        await fs.writeFile(path.join(artifactsDir, "page-source.html"), await browser.getPageSource(), "utf8");
        await browser.saveScreenshot(path.join(artifactsDir, "failure.png"));
      } catch {
        // session 创建失败时保留应用 stdout/stderr 和 manifest。
      }
    }
  },
  onComplete: async () => {
    if (app) await stopWebDriverApp(app);
    if (!process.env.COPYPOLISH_E2E_KEEP_SETTINGS && process.env.COPYPOLISH_E2E_SETTINGS_DIR) {
      await fs.rm(process.env.COPYPOLISH_E2E_SETTINGS_DIR, { recursive: true, force: true });
    }
  },
};