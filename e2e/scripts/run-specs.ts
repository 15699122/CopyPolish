import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const e2eDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const wdioCli = path.join(e2eDir, "node_modules", "@wdio", "cli", "bin", "wdio.js");
const specsDir = path.join(e2eDir, "specs");
const extraArgs = process.argv.slice(2);

const requestedSpec = extraArgs.indexOf("--spec");
const specs = requestedSpec >= 0 && extraArgs[requestedSpec + 1]
  ? [path.resolve(e2eDir, extraArgs[requestedSpec + 1])]
  : fs.readdirSync(specsDir)
      .filter((name) => name.endsWith(".spec.ts"))
      .sort()
      .map((name) => path.join(specsDir, name));

if (specs.length === 0) {
  throw new Error(`No E2E spec files found in ${specsDir}`);
}

for (const spec of specs) {
  const settingsDir = fs.mkdtempSync(path.join(e2eDir, "settings-"));
  const args = [wdioCli, "run", path.join(e2eDir, "wdio.conf.ts"), "--spec", spec, ...extraArgs.filter((_, index) => index !== requestedSpec && index !== requestedSpec + 1)];
  console.log(`\n=== Running isolated E2E spec: ${path.relative(e2eDir, spec)} ===`);

  try {
    const result = spawnSync(process.execPath, args, {
      cwd: e2eDir,
      env: {
        ...process.env,
        COPYPOLISH_E2E_SETTINGS_DIR: settingsDir,
      },
      stdio: "inherit",
    });

    if (result.error) throw result.error;
    if (result.status !== 0) {
      process.exitCode = result.status ?? 1;
      break;
    }
  } finally {
    fs.rmSync(settingsDir, { recursive: true, force: true });
  }
}
