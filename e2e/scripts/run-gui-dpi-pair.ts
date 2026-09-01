import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const e2eDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const runner = path.join(e2eDir, "scripts", "run-gui-visual-artifacts.ts");
const tsxCli = path.join(e2eDir, "node_modules", "tsx", "dist", "cli.mjs");
const scaleIndex = process.argv.findIndex((arg) => arg === "--expected-scale");
const scale = Number(scaleIndex >= 0 ? process.argv[scaleIndex + 1] : "0");
if (scaleIndex >= 0 && (!Number.isFinite(scale) || scale <= 0)) {
  throw new Error("用法：npm run test:gui-dpi [-- --expected-scale N]");
}
for (const webdriver of [false, true]) {
  const result = spawnSync(process.execPath, [
    tsxCli,
    runner,
    "--expected-scale",
    String(scale),
    ...(webdriver ? ["--webdriver"] : []),
  ], {
    cwd: e2eDir,
    env: process.env,
    stdio: "inherit",
  });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}
console.log(scale > 0 ? `GUI DPI pair recorded: embedded/webdriver at ${scale}%` : "GUI DPI pair recorded: embedded/webdriver at detected scale");