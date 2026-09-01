import { createHash } from "node:crypto";
import { execFileSync, spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  environmentSummary,
  prepareArtifactDir,
  writeArtifactJson,
} from "../support/artifacts.js";

const e2eDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const rootDir = path.resolve(e2eDir, "..");
const prepareOnly = process.argv.includes("--prepare-only");
const artifactDir = path.join(
  e2eDir,
  "artifacts",
  "windows-terminal-tui",
  String(Date.now()),
);
const binaryPath = process.env.COPYPOLISH_TUI_BINARY
  ?? [
    path.join(rootDir, "src-tauri", "target", "release", "copypolish-tui.exe"),
    path.join(rootDir, "src-tauri", "target", "debug", "copypolish-tui.exe"),
  ].find((candidate) => fs.existsSync(candidate));

function safeOutput(command: string, args: string[]): string | null {
  try {
    return execFileSync(command, args, {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
      windowsHide: true,
    }).trim();
  } catch {
    return null;
  }
}

function clipboardHash(): string | null {
  const value = safeOutput("pwsh.exe", [
    "-NoProfile",
    "-Command",
    "$value = Get-Clipboard -Raw -ErrorAction SilentlyContinue; if ($null -ne $value) { [Console]::Out.Write($value) }",
  ]);
  return value === null ? null : createHash("sha256").update(value, "utf8").digest("hex");
}

if (process.platform !== "win32") {
  throw new Error("Windows Terminal TUI artifact runner 仅支持 Windows");
}
if (!binaryPath) {
  throw new Error("TUI binary not found; build copypolish-tui or set COPYPOLISH_TUI_BINARY");
}

await prepareArtifactDir(artifactDir);
const terminalEnvironment = {
  ...environmentSummary(),
  wtSession: process.env.WT_SESSION ?? null,
  wtProfileId: process.env.WT_PROFILE_ID ?? null,
  termProgram: process.env.TERM_PROGRAM ?? null,
  columns: process.stdout.columns ?? null,
  rows: process.stdout.rows ?? null,
  windowsTerminal: safeOutput("wt.exe", ["--version"]),
  powershell: safeOutput("pwsh.exe", ["--version"]),
  codePage: safeOutput("cmd.exe", ["/d", "/c", "chcp"]),
};
await writeArtifactJson(artifactDir, "manifest.json", {
  schemaVersion: 1,
  artifact: "windows-terminal-tui",
  binaryPath,
  prepareOnly,
  environment: terminalEnvironment,
});
await writeArtifactJson(artifactDir, "manual-checklist.json", {
  schemaVersion: 1,
  cases: [
    { id: "raw-mode", status: "pending" },
    { id: "multiline-wrap-wt-tui-001", status: "pending" },
    { id: "cursor-visible-wt-tui-002", status: "pending" },
    { id: "emoji-wt-tui-003", status: "pending" },
    { id: "delete-backspace-grapheme", status: "pending" },
    { id: "bracketed-paste", status: "pending" },
    { id: "osc52-clipboard", status: "pending" },
    { id: "settings-save-restart", status: "pending" },
    { id: "terminal-restored-after-exit", status: "pending" },
  ],
  evidenceRequired: [
    "screenshots/*.png",
    "按键序列",
    "Windows Terminal profile/font/size",
    "每项实际结果与预期结果",
  ],
});

if (prepareOnly) {
  await writeArtifactJson(artifactDir, "result.json", {
    schemaVersion: 1,
    status: "prepared",
    exitCode: 0,
    manualConfirmationRequired: true,
  });
  console.log(`Windows Terminal TUI artifact prepared: ${artifactDir}`);
  process.exit(0);
}

if (!process.env.WT_SESSION) {
  throw new Error("未检测到 WT_SESSION；必须从真实 Windows Terminal 中运行此入口");
}

const clipboardBefore = clipboardHash();
const result = spawnSync(binaryPath, [], {
  cwd: rootDir,
  env: process.env,
  stdio: "inherit",
});
const clipboardAfter = clipboardHash();
await writeArtifactJson(artifactDir, "result.json", {
  schemaVersion: 1,
  status: "manual-confirmation-required",
  exitCode: result.status,
  signal: result.signal,
  clipboardBeforeSha256: clipboardBefore,
  clipboardAfterSha256: clipboardAfter,
  clipboardChanged: clipboardBefore !== null
    && clipboardAfter !== null
    && clipboardBefore !== clipboardAfter,
  manualConfirmationRequired: true,
});
if (result.error) throw result.error;
console.log(`Windows Terminal TUI run finished; complete checklist/screenshots in: ${artifactDir}`);