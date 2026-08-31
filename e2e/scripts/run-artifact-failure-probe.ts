import { spawnSync } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";

const e2eDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const wdioCli = path.join(e2eDir, "node_modules", "@wdio", "cli", "bin", "wdio.js");
const useWebdriver = process.argv.includes("--webdriver");
const config = useWebdriver ? "wdio.webdriver.conf.ts" : "wdio.conf.ts";
const spec = path.join(e2eDir, "specs", "artifact-failure-probe.spec.ts");

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

async function hasFile(directory: string, predicate: (file: string) => boolean): Promise<boolean> {
  try {
    const entries = await fs.promises.readdir(directory, { withFileTypes: true });
    return entries.some((entry) => entry.isFile() && predicate(entry.name));
  } catch {
    return false;
  }
}

async function hasNestedFile(directory: string): Promise<boolean> {
  try {
    const entries = await fs.promises.readdir(directory, { withFileTypes: true });
    for (const entry of entries) {
      const entryPath = path.join(directory, entry.name);
      if (entry.isFile()) return true;
      if (entry.isDirectory() && await hasNestedFile(entryPath)) return true;
    }
  } catch {
    return false;
  }
  return false;
}

async function hasLogFile(directory: string): Promise<boolean> {
  try {
    const entries = await fs.promises.readdir(directory, { withFileTypes: true });
    for (const entry of entries) {
      const entryPath = path.join(directory, entry.name);
      if (entry.isFile() && entry.name.endsWith(".log")) return true;
      if (entry.isDirectory() && await hasLogFile(entryPath)) return true;
    }
  } catch {
    return false;
  }
  return false;
}

async function validateArtifacts(artifactDir: string): Promise<void> {
  const manifestPath = path.join(artifactDir, "manifest.json");
  const resultPath = path.join(artifactDir, "result.json");
  const fixturePath = path.join(artifactDir, "settings-fixture", "rules.yaml");

  const manifest = JSON.parse(await fs.promises.readFile(manifestPath, "utf8")) as {
    schemaVersion?: number;
    provider?: string;
  };
  const result = JSON.parse(await fs.promises.readFile(resultPath, "utf8")) as {
    exitCode?: number;
    results?: { failed?: number };
  };

  if (manifest.schemaVersion !== 1) throw new Error("artifact manifest schemaVersion is invalid");
  if (manifest.provider !== (useWebdriver ? "webdriver" : "embedded")) {
    throw new Error(`artifact manifest provider is invalid: ${manifest.provider}`);
  }
  if (result.exitCode === 0 || (result.results?.failed ?? 0) < 1) {
    throw new Error("artifact result does not record the expected WDIO failure");
  }
  if (!await hasFile(path.join(artifactDir, "screenshots"), (file) => file.endsWith(".png"))) {
    throw new Error("artifact screenshot is missing");
  }
  if (!await hasFile(artifactDir, (file) => file.endsWith(".html"))) {
    throw new Error("artifact page source is missing");
  }
  if (!await hasLogFile(artifactDir)) {
    throw new Error("artifact WDIO/provider logs are missing");
  }
  await fs.promises.access(fixturePath);
}

const settingsDir = fs.mkdtempSync(path.join(e2eDir, "settings-artifact-probe-"));
const artifactDir = path.join(
  e2eDir,
  "artifacts",
  useWebdriver ? "webdriver" : "embedded",
  `${Date.now()}-artifact-failure-probe`,
);
const settingsPath = path.join(settingsDir, "rules.yaml");
fs.writeFileSync(
  settingsPath,
  "enabled:\n  - spacing.cjk-latin\n  - spacing.cjk-number\nlast_input: \"\"\n",
  "utf8",
);
const port = useWebdriver ? await findFreePort() : undefined;

try {
  const args = [wdioCli, "run", path.join(e2eDir, config), "--spec", spec];
  console.log(`\n=== Running ${useWebdriver ? "W3C" : "embedded"} artifact failure probe ===`);
  const result = spawnSync(process.execPath, args, {
    cwd: e2eDir,
    env: {
      ...process.env,
      COPYPOLISH_E2E_ARTIFACT_PROBE: "1",
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
  if (result.status === 0) throw new Error("artifact failure probe unexpectedly passed");

  await validateArtifacts(artifactDir);
  console.log("Expected test failure observed.");
  console.log("Artifact bundle validation passed.");
} finally {
  fs.rmSync(settingsDir, { recursive: true, force: true });
}