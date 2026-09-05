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
const extraFeatures = process.argv
  .flatMap((argument, index, argumentsList) => argument === "--feature" ? [argumentsList[index + 1]] : [])
  .filter((feature): feature is string => Boolean(feature));

const tauriConfig = JSON.parse(
  await fs.readFile(path.join(tauriDir, "tauri.conf.json"), "utf8"),
) as {
  app?: {
    security?: Record<string, unknown>;
    [key: string]: unknown;
  };
};

// 每个 provider 都必须重新生成匹配自身的 frontend/dist，避免先执行
// tauri-plugin-webdriver 构建后，旧 embedded provider 复用错误的 bundle。
await execFileAsync(process.execPath, [npmCliPath, "run", "build", "--prefix", path.join(rootDir, "frontend")], {
  cwd: rootDir,
  env: { ...process.env, VITE_COPYPOLISH_E2E: "true" },
  maxBuffer: 10 * 1024 * 1024,
});

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

const cargoFeatures = ["e2e", ...extraFeatures].join(",");
await execFileAsync("cargo", ["build", "--manifest-path", "Cargo.toml", "--features", cargoFeatures], {
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
if (extraFeatures.length > 0) console.log(`Additional Cargo features: ${extraFeatures.join(", ")}`);