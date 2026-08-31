import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { environmentSummary, prepareArtifactDir, writeArtifactJson } from "../support/artifacts.js";

const e2eDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const rootDir = path.resolve(e2eDir, "..");
const artifactDir = path.join(e2eDir, "artifacts", "tui-transcript", `${Date.now()}`);
const binaryPath = process.env.COPYPOLISH_TUI_BINARY
  ?? [
    path.join(rootDir, "src-tauri", "target", "release", process.platform === "win32" ? "copypolish-tui.exe" : "copypolish-tui"),
    path.join(rootDir, "src-tauri", "target", "debug", process.platform === "win32" ? "copypolish-tui.exe" : "copypolish-tui"),
  ].find((candidate) => fs.existsSync(candidate));

type TranscriptCase = {
  name: string;
  args: string[];
  input?: string;
  expectedExitCode: number;
  expectedStdout?: string;
  expectedStderr?: string;
};

type TranscriptResult = TranscriptCase & {
  command: string[];
  exitCode: number | null;
  signal: NodeJS.Signals | null;
  stdout: string;
  stderr: string;
  passed: boolean;
};

if (!binaryPath) {
  throw new Error("TUI binary not found; build copypolish-tui or set COPYPOLISH_TUI_BINARY");
}

const missingInput = path.join(artifactDir, "missing-input.txt");
const cases: TranscriptCase[] = [
  {
    name: "default-formatting",
    args: ["--stdin", "--no-config"],
    input: "在LeanCloud上，花了5000元",
    expectedExitCode: 0,
    expectedStdout: "在 LeanCloud 上，花了 5000 元\n",
  },
  {
    name: "none-is-identity",
    args: ["--stdin", "--no-config", "--rules", "none"],
    input: "在LeanCloud上，花了5000元",
    expectedExitCode: 0,
    expectedStdout: "在LeanCloud上，花了5000元\n",
  },
  {
    name: "unknown-rule-warning",
    args: ["--stdin", "--no-config", "--enable", "unknown-rule-key"],
    input: "测试",
    expectedExitCode: 0,
    expectedStderr: "警告：未知的规则 key：unknown-rule-key",
  },
  {
    name: "missing-input-error",
    args: ["--input", missingInput, "--no-config"],
    expectedExitCode: 1,
    expectedStderr: "读取输入文件",
  },
];

function runCase(testCase: TranscriptCase): TranscriptResult {
  const result = spawnSync(binaryPath!, testCase.args, {
    cwd: rootDir,
    input: testCase.input,
    encoding: "utf8",
    windowsHide: true,
  });
  const stdout = result.stdout ?? "";
  const stderr = result.stderr ?? "";
  const passed = result.status === testCase.expectedExitCode
    && (testCase.expectedStdout === undefined || stdout === testCase.expectedStdout)
    && (testCase.expectedStderr === undefined || stderr.includes(testCase.expectedStderr));
  return {
    ...testCase,
    command: [binaryPath!, ...testCase.args],
    exitCode: result.status,
    signal: result.signal,
    stdout,
    stderr,
    passed,
  };
}

await prepareArtifactDir(artifactDir);
const results = cases.map(runCase);
for (const result of results) {
  await fs.promises.writeFile(path.join(artifactDir, `${result.name}.stdin.txt`), result.input ?? "", "utf8");
  await fs.promises.writeFile(path.join(artifactDir, `${result.name}.stdout.txt`), result.stdout, "utf8");
  await fs.promises.writeFile(path.join(artifactDir, `${result.name}.stderr.txt`), result.stderr, "utf8");
  await writeArtifactJson(artifactDir, `${result.name}.json`, {
    command: result.command,
    expectedExitCode: result.expectedExitCode,
    exitCode: result.exitCode,
    signal: result.signal,
    passed: result.passed,
  });
}

await writeArtifactJson(artifactDir, "manifest.json", {
  schemaVersion: 1,
  artifact: "tui-transcript",
  binaryPath,
  environment: environmentSummary(),
  cases: cases.map(({ name }) => name),
});
await writeArtifactJson(artifactDir, "result.json", {
  schemaVersion: 1,
  status: results.every((result) => result.passed) ? "completed" : "failed",
  exitCode: results.every((result) => result.passed) ? 0 : 1,
  passed: results.filter((result) => result.passed).length,
  failed: results.filter((result) => !result.passed).length,
});

if (!results.every((result) => result.passed)) {
  throw new Error(`TUI transcript failed; inspect ${artifactDir}`);
}

console.log(`TUI transcript passed: ${results.length}/${results.length}`);
console.log(`Artifact: ${artifactDir}`);