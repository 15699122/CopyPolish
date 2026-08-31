import { spawnSync } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { SETTINGS_FIXTURES, type SettingsFixture } from "../support/settings-fixtures.js";

const e2eDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const wdioCli = path.join(e2eDir, "node_modules", "@wdio", "cli", "bin", "wdio.js");
const useWebdriver = process.argv.includes("--webdriver");
const config = useWebdriver ? "wdio.webdriver.conf.ts" : "wdio.conf.ts";
const spec = path.join(e2eDir, "specs", "corrupt-settings.spec.ts");

async function findFreePort(): Promise<number> {
  return await new Promise((resolve, reject) => {
    const server = net.createServer();
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (!address || typeof address === "string") {
        server.close();
        reject(new Error("Unable to allocate a WebDriver port"));
        return;
      }
      server.close(() => resolve(address.port));
    });
  });
}

for (const fixture of SETTINGS_FIXTURES as readonly SettingsFixture[]) {
  const settingsDir = fs.mkdtempSync(path.join(e2eDir, "settings-corrupt-"));
  const artifactDir = path.join(
    e2eDir,
    "artifacts",
    useWebdriver ? "webdriver" : "embedded",
    `${Date.now()}-corrupt-settings-${fixture}`,
  );
  const port = useWebdriver ? await findFreePort() : undefined;
  const args = [wdioCli, "run", path.join(e2eDir, config), "--spec", spec];

  console.log(`\n=== Running ${useWebdriver ? "W3C" : "embedded"} corrupt-settings fixture: ${fixture} ===`);
  const result = spawnSync(process.execPath, args, {
    cwd: e2eDir,
    env: {
      ...process.env,
      COPYPOLISH_E2E_SETTINGS_DIR: settingsDir,
      COPYPOLISH_E2E_SETTINGS_FIXTURE: fixture,
      ...(useWebdriver
        ? {
            TAURI_WEBDRIVER_PORT: String(port),
            COPYPOLISH_E2E_ARTIFACT_DIR: artifactDir,
            VITE_COPYPOLISH_E2E_PROVIDER: "webdriver",
          }
        : {}),
    },
    stdio: "inherit",
  });

  if (result.error) throw result.error;
  if (result.status !== 0) {
    if (!process.env.COPYPOLISH_E2E_KEEP_SETTINGS) fs.rmSync(settingsDir, { recursive: true, force: true });
    process.exit(result.status ?? 1);
  }
  if (!process.env.COPYPOLISH_E2E_KEEP_SETTINGS) fs.rmSync(settingsDir, { recursive: true, force: true });
}