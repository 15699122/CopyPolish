import path from "node:path";
import fs from "node:fs";
import { fileURLToPath } from "node:url";
import {
  captureBrowserFailure,
  copySettingsFixture,
  prepareArtifactDir,
  writeManifest,
  writeResult,
} from "./support/artifacts.js";
import { prepareSettingsFixture } from "./support/settings-fixtures.js";

const e2eDir = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(e2eDir, "..");
const artifactsDir = process.env.COPYPOLISH_E2E_ARTIFACT_DIR
  ?? path.join(e2eDir, "artifacts", "embedded", String(Date.now()));
const settingsDir = process.env.COPYPOLISH_E2E_SETTINGS_DIR ?? fs.mkdtempSync(path.join(e2eDir, "settings-"));
process.env.COPYPOLISH_E2E_SETTINGS_DIR = settingsDir;
prepareSettingsFixture(
  settingsDir,
  process.env.COPYPOLISH_E2E_SETTINGS_FIXTURE as Parameters<typeof prepareSettingsFixture>[1],
);
const binaryPath = process.platform === "win32"
  ? path.join(rootDir, "src-tauri", "target", "debug", "chinese-copywriting-formatter.exe")
  : path.join(rootDir, "src-tauri", "target", "debug", "chinese-copywriting-formatter");

export const config: WebdriverIO.Config = {
  runner: "local",
  specs: ["./specs/**/*.spec.ts"],
  maxInstances: 1,
  maxInstancesPerCapability: 1,
  framework: "mocha",
  reporters: ["spec"],
  services: [
    [
      "@wdio/tauri-service",
      {
        appBinaryPath: binaryPath,
        driverProvider: "embedded",
        embeddedPort: Number(process.env.TAURI_WEBDRIVER_PORT ?? 4445),
        env: {
          VITE_COPYPOLISH_E2E: "true",
          // WSLg/软件渲染环境中避免 WebKitGTK 选择不可用的 Zink GPU 后端。
          GDK_BACKEND: process.env.GDK_BACKEND ?? "x11",
          LIBGL_ALWAYS_SOFTWARE: process.env.LIBGL_ALWAYS_SOFTWARE ?? "1",
          MESA_LOADER_DRIVER_OVERRIDE: process.env.MESA_LOADER_DRIVER_OVERRIDE ?? "llvmpipe",
          ...(process.env.COPYPOLISH_E2E_SETTINGS_DIR
            ? { COPYPOLISH_E2E_SETTINGS_DIR: process.env.COPYPOLISH_E2E_SETTINGS_DIR }
            : {}),
        },
        captureBackendLogs: true,
        captureFrontendLogs: true,
        logLevel: "info",
        backendLogLevel: "debug",
        frontendLogLevel: "debug",
        logDir: path.join(artifactsDir, "logs"),
        startTimeout: 30_000,
        commandTimeout: 30_000,
      },
    ],
  ],
  capabilities: [
    {
      browserName: "tauri",
      "wdio:maxInstances": 1,
    },
  ],
  mochaOpts: {
    timeout: 60_000,
  },
  logLevel: "info",
  outputDir: path.join(artifactsDir, "wdio"),
  afterTest: async (_test, _context, result) => {
    if (!result.passed) {
      await captureBrowserFailure(artifactsDir, `${Date.now()}-failure`);
      await copySettingsFixture(settingsDir, artifactsDir);
    }
  },
  afterHook: async (_test, _context, result, hookName) => {
    if (hookName.includes("before all") && !result.passed) {
      await captureBrowserFailure(artifactsDir, `${Date.now()}-hook-failure`);
      await copySettingsFixture(settingsDir, artifactsDir);
    }
  },
  onComplete: async (exitCode, _config, _capabilities, results) => {
    await prepareArtifactDir(artifactsDir);
    if (exitCode !== 0) await copySettingsFixture(settingsDir, artifactsDir);
    await writeResult(artifactsDir, {
      exitCode,
      status: exitCode === 0 ? "completed" : "failed",
      results,
    });
    if (!process.env.COPYPOLISH_E2E_KEEP_SETTINGS) {
      await fs.promises.rm(settingsDir, { recursive: true, force: true });
    }
  },
};

await writeManifest({
  artifactDir: artifactsDir,
  provider: "embedded",
  settingsDir,
  binaryPath,
  port: Number(process.env.TAURI_WEBDRIVER_PORT ?? 4445),
});