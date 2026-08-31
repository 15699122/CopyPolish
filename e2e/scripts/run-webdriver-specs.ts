import { spawnSync } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";

const e2eDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const wdioCli = path.join(e2eDir, "node_modules", "@wdio", "cli", "bin", "wdio.js");
const specsDir = path.join(e2eDir, "specs");
const extraArgs = process.argv.slice(2);

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

const requestedSpec = extraArgs.indexOf("--spec");
const specs = requestedSpec >= 0 && extraArgs[requestedSpec + 1]
  ? [path.resolve(e2eDir, extraArgs[requestedSpec + 1])]
  : fs.readdirSync(specsDir)
      .filter((name) => name.endsWith(".spec.ts"))
      .sort()
      .map((name) => path.join(specsDir, name));

if (specs.length === 0) throw new Error(`No E2E spec files found in ${specsDir}`);

for (const spec of specs) {
  const settingsDir = fs.mkdtempSync(path.join(e2eDir, "settings-webdriver-"));
  const artifactDir = path.join(e2eDir, "artifacts", "webdriver", `${Date.now()}-${path.basename(spec, ".spec.ts")}`);
  const port = await findFreePort();
  const args = [
    wdioCli,
    "run",
    path.join(e2eDir, "wdio.webdriver.conf.ts"),
    "--spec",
    spec,
    ...extraArgs.filter((_, index) => index !== requestedSpec && index !== requestedSpec + 1),
  ];
  console.log(`\n=== Running isolated WebDriver provider spec: ${path.relative(e2eDir, spec)} (port ${port}) ===`);

  const result = spawnSync(process.execPath, args, {
    cwd: e2eDir,
    env: {
      ...process.env,
      TAURI_WEBDRIVER_PORT: String(port),
      COPYPOLISH_E2E_SETTINGS_DIR: settingsDir,
      COPYPOLISH_E2E_ARTIFACT_DIR: artifactDir,
      VITE_COPYPOLISH_E2E_PROVIDER: "webdriver",
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