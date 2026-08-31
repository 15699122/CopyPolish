import { execFile } from "node:child_process";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const e2eDir = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(e2eDir, "../..");
const tauriDir = path.join(rootDir, "src-tauri");

const tauriConfig = JSON.parse(
  await fs.readFile(path.join(tauriDir, "tauri.conf.json"), "utf8"),
) as {
  app?: {
    security?: Record<string, unknown>;
    [key: string]: unknown;
  };
};
tauriConfig.app = { ...tauriConfig.app, withGlobalTauri: true };
tauriConfig.app.security = {
  ...tauriConfig.app.security,
  capabilities: ["default", "e2e"],
};

// Tauri 将 frontendDist 嵌入 binary；Cargo 不会仅因 dist 文件内容变化而
// 自动重新触发所有资源生成，因此 E2E 构建必须清理应用包，避免 smoke
// 实际运行到上一轮前端资源。
await execFileAsync("cargo", ["clean", "-p", "chinese-copywriting-formatter"], {
  cwd: tauriDir,
  maxBuffer: 10 * 1024 * 1024,
});

await execFileAsync("cargo", ["build", "--manifest-path", "Cargo.toml", "--features", "e2e"], {
  cwd: tauriDir,
  env: { ...process.env, TAURI_CONFIG: JSON.stringify(tauriConfig) },
  maxBuffer: 10 * 1024 * 1024,
});

const binaryName = process.platform === "win32"
  ? "chinese-copywriting-formatter.exe"
  : "chinese-copywriting-formatter";
const binaryPath = path.join(tauriDir, "target", "debug", binaryName);
await fs.access(binaryPath);
console.log(`E2E binary ready: ${binaryPath}`);