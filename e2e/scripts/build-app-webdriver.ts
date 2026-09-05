import { execFile } from "node:child_process";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const e2eDir = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(e2eDir, "../..");
const tauriDir = path.join(rootDir, "src-tauri");
const npmCliPath = process.env.npm_execpath;
if (!npmCliPath) throw new Error("npm_execpath is unavailable; run this script through npm");

await execFileAsync(process.execPath, [npmCliPath, "run", "build", "--prefix", path.join(rootDir, "frontend")], {
  cwd: rootDir,
  env: {
    ...process.env,
    VITE_COPYPOLISH_E2E: "true",
    VITE_COPYPOLISH_E2E_PROVIDER: "webdriver",
  },
  maxBuffer: 10 * 1024 * 1024,
});

await execFileAsync("cargo", ["clean", "-p", "chinese-copywriting-formatter"], {
  cwd: tauriDir,
  maxBuffer: 10 * 1024 * 1024,
});

await execFileAsync("cargo", [
  "build",
  "--manifest-path",
  "Cargo.toml",
  "--features",
  "e2e-webdriver",
], {
  cwd: tauriDir,
  env: { ...process.env },
  maxBuffer: 10 * 1024 * 1024,
});

const binaryName = process.platform === "win32"
  ? "chinese-copywriting-formatter.exe"
  : "chinese-copywriting-formatter";
const binaryPath = path.join(tauriDir, "target", "debug", binaryName);
await fs.access(binaryPath);
console.log(`WebDriver provider binary ready: ${binaryPath}`);