import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  captureBrowserFailure,
  copySettingsFixture,
  prepareArtifactDir,
  writeManifest,
  writeResult,
} from "./support/artifacts.js";
import { startWebDriverApp, stopWebDriverApp, type WebDriverApp } from "./support/webdriver-app.js";
import { prepareSettingsFixture } from "./support/settings-fixtures.js";

const e2eDir = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(e2eDir, "..");
const artifactsDir = process.env.COPYPOLISH_E2E_ARTIFACT_DIR
  ?? path.join(e2eDir, "artifacts", "webdriver");
const port = Number(process.env.TAURI_WEBDRIVER_PORT ?? 4445);
const binaryPath = path.join(
  rootDir,
  "src-tauri",
  "target",
  "debug",
  process.platform === "win32" ? "chinese-copywriting-formatter.exe" : "chinese-copywriting-formatter",
);
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
    await writeManifest({
      artifactDir: artifactsDir,
      provider: "webdriver",
      settingsDir: process.env.COPYPOLISH_E2E_SETTINGS_DIR,
      binaryPath,
      port,
      pid: app?.pid,
    });
  },
  afterTest: async (_test, _context, result) => {
    if (!result.passed) {
      await captureBrowserFailure(artifactsDir, "failure");
      await copySettingsFixture(process.env.COPYPOLISH_E2E_SETTINGS_DIR, artifactsDir);
    }
  },
  onComplete: async (exitCode, _config, _capabilities, results) => {
    if (app) await stopWebDriverApp(app);
    await prepareArtifactDir(artifactsDir);
    if (exitCode !== 0) {
      await copySettingsFixture(process.env.COPYPOLISH_E2E_SETTINGS_DIR, artifactsDir);
    }
    await writeResult(artifactsDir, {
      exitCode,
      status: exitCode === 0 ? "completed" : "failed",
      results,
    });
    if (!process.env.COPYPOLISH_E2E_KEEP_SETTINGS && process.env.COPYPOLISH_E2E_SETTINGS_DIR) {
      await fs.rm(process.env.COPYPOLISH_E2E_SETTINGS_DIR, { recursive: true, force: true });
    }
  },
};