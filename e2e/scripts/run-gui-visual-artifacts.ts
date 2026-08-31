import { spawnSync } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";

const e2eDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const wdioCli = path.join(e2eDir, "node_modules", "@wdio", "cli", "bin", "wdio.js");
const useWebdriver = process.argv.includes("--webdriver");
const config = useWebdriver ? "wdio.webdriver.conf.ts" : "wdio.conf.ts";
const spec = path.join(e2eDir, "specs", "gui-visual-artifacts.spec.ts");
const stateNames = ["main-normal", "settings-light", "settings-dark", "main-narrow", "settings-narrow"];

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
  const visualStates = JSON.parse(await fs.promises.readFile(path.join(artifactDir, "visual-states.json"), "utf8")) as {
    schemaVersion?: number;
    states?: unknown[];
  };
  if (visualStates.schemaVersion !== 1 || visualStates.states?.length !== stateNames.length) {
    throw new Error("visual-states.json is incomplete");
  }

  for (const name of stateNames) {
    await fs.promises.access(path.join(artifactDir, `${name}.html`));
    await fs.promises.access(path.join(artifactDir, `${name}.json`));
    await fs.promises.access(path.join(artifactDir, "screenshots", `${name}.png`));
  }
}

const settingsDir = fs.mkdtempSync(path.join(e2eDir, "settings-gui-visual-"));
const artifactDir = path.join(
  e2eDir,
  "artifacts",
  useWebdriver ? "webdriver" : "embedded",
  `${Date.now()}-gui-visual-artifacts`,
);
fs.writeFileSync(path.join(settingsDir, "rules.yaml"), "enabled: []\nlast_input: \"\"\n", "utf8");
const port = useWebdriver ? await findFreePort() : undefined;

try {
  const args = [wdioCli, "run", path.join(e2eDir, config), "--spec", spec];
  console.log(`\n=== Running ${useWebdriver ? "W3C" : "embedded"} GUI visual artifact spec ===`);
  const result = spawnSync(process.execPath, args, {
    cwd: e2eDir,
    env: {
      ...process.env,
      COPYPOLISH_E2E_VISUAL_ARTIFACTS: "1",
      COPYPOLISH_E2E_KEEP_SETTINGS: "1",
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
  if (result.status === 0) await validateArtifacts(artifactDir);
} finally {
  fs.rmSync(settingsDir, { recursive: true, force: true });
}