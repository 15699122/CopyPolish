import { spawnSync } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";

const e2eDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const wdioCli = path.join(e2eDir, "node_modules", "@wdio", "cli", "bin", "wdio.js");
const useWebdriver = process.argv.includes("--webdriver");
const config = useWebdriver ? "wdio.webdriver.conf.ts" : "wdio.conf.ts";
const spec = path.join(e2eDir, "specs", "settings-shortcut-console.spec.ts");

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

async function validateArtifacts(artifactDir: string): Promise<void> {
  const summary = JSON.parse(
    await fs.promises.readFile(path.join(artifactDir, "shortcut-console-summary.json"), "utf8"),
  ) as { schemaVersion?: number; settingsOpened?: boolean; actWarningCount?: number };
  if (summary.schemaVersion !== 1 || !summary.settingsOpened || summary.actWarningCount !== 0) {
    throw new Error("settings shortcut console summary is invalid");
  }
  await fs.promises.access(path.join(artifactDir, "console-events.json"));
  await fs.promises.access(path.join(artifactDir, "settings-shortcut-after-key.html"));
  await fs.promises.access(path.join(artifactDir, "screenshots", "settings-shortcut-after-key.png"));
}

const settingsDir = fs.mkdtempSync(path.join(e2eDir, "settings-shortcut-console-"));
const artifactDir = path.join(
  e2eDir,
  "artifacts",
  useWebdriver ? "webdriver" : "embedded",
  `${Date.now()}-settings-shortcut-console`,
);
fs.writeFileSync(path.join(settingsDir, "rules.yaml"), "enabled: []\nlast_input: \"\"\n", "utf8");
const port = useWebdriver ? await findFreePort() : undefined;

try {
  const result = spawnSync(process.execPath, [
    wdioCli,
    "run",
    path.join(e2eDir, config),
    "--spec",
    spec,
  ], {
    cwd: e2eDir,
    env: {
      ...process.env,
      COPYPOLISH_E2E_SHORTCUT_CONSOLE: "1",
      COPYPOLISH_E2E_SETTINGS_DIR: settingsDir,
      COPYPOLISH_E2E_ARTIFACT_DIR: artifactDir,
      ...(useWebdriver
        ? {
            TAURI_WEBDRIVER_PORT: String(port),
            VITE_COPYPOLISH_E2E_PROVIDER: "webdriver",
          }
        : {}),
    },
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exitCode = result.status ?? 1;
  if (result.status === 0) {
    await validateArtifacts(artifactDir);
    console.log(`Settings shortcut console artifact: ${artifactDir}`);
  }
} finally {
  fs.rmSync(settingsDir, { recursive: true, force: true });
}