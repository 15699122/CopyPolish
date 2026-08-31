import fs from "node:fs/promises";
import { execFileSync } from "node:child_process";
import path from "node:path";

export type ArtifactProvider = "embedded" | "webdriver";

export type ArtifactContext = {
  artifactDir: string;
  provider: ArtifactProvider;
  settingsDir?: string;
  binaryPath?: string;
  port?: number;
  pid?: number;
};

type JsonRecord = Record<string, unknown>;

export async function prepareArtifactDir(artifactDir: string): Promise<void> {
  await Promise.all([
    fs.mkdir(path.join(artifactDir, "logs"), { recursive: true }),
    fs.mkdir(path.join(artifactDir, "screenshots"), { recursive: true }),
    fs.mkdir(path.join(artifactDir, "wdio"), { recursive: true }),
  ]);
}

function safeVersion(command: string): string | null {
  try {
    return execFileSync(command, ["--version"], {
      encoding: "utf8",
      stdio: ["ignore", "pipe", "ignore"],
      windowsHide: true,
    }).trim();
  } catch {
    return null;
  }
}

export function environmentSummary(): JsonRecord {
  return {
    platform: process.platform,
    arch: process.arch,
    node: process.version,
    rustc: safeVersion(process.platform === "win32" ? "rustc.exe" : "rustc"),
    cargo: safeVersion(process.platform === "win32" ? "cargo.exe" : "cargo"),
    commit: process.env.CI_COMMIT_SHA ?? process.env.GITHUB_SHA ?? "local",
    provider: process.env.VITE_COPYPOLISH_E2E_PROVIDER ?? "embedded",
  };
}

export async function writeArtifactJson(
  artifactDir: string,
  fileName: string,
  value: JsonRecord,
): Promise<void> {
  await fs.writeFile(
    path.join(artifactDir, fileName),
    `${JSON.stringify(value, null, 2)}\n`,
    "utf8",
  );
}

export async function writeManifest(context: ArtifactContext): Promise<void> {
  await prepareArtifactDir(context.artifactDir);
  let previous: JsonRecord = {};
  try {
    previous = JSON.parse(await fs.readFile(path.join(context.artifactDir, "manifest.json"), "utf8")) as JsonRecord;
  } catch {
    // 首次写入时从空对象开始。
  }
  await writeArtifactJson(context.artifactDir, "manifest.json", {
    ...previous,
    schemaVersion: 1,
    provider: context.provider,
    binaryPath: context.binaryPath ?? previous.binaryPath ?? null,
    settingsDir: context.settingsDir ?? previous.settingsDir ?? null,
    port: context.port ?? previous.port ?? null,
    pid: context.pid ?? previous.pid ?? null,
    startedAt: previous.startedAt ?? new Date().toISOString(),
    environment: environmentSummary(),
  });
}

export async function writeResult(
  artifactDir: string,
  result: JsonRecord,
): Promise<void> {
  let previous: JsonRecord = {};
  try {
    previous = JSON.parse(await fs.readFile(path.join(artifactDir, "result.json"), "utf8")) as JsonRecord;
  } catch {
    // 首次写入或文件损坏时从空结果开始，不能阻断原始测试结果。
  }
  await writeArtifactJson(artifactDir, "result.json", {
    ...previous,
    schemaVersion: 1,
    finishedAt: new Date().toISOString(),
    ...result,
  });
}

export async function copySettingsFixture(
  settingsDir: string | undefined,
  artifactDir: string,
): Promise<void> {
  if (!settingsDir) return;

  try {
    await fs.access(settingsDir);
  } catch {
    return;
  }

  try {
    const destination = path.join(artifactDir, "settings-fixture");
    await fs.rm(destination, { recursive: true, force: true });
    await fs.cp(settingsDir, destination, { recursive: true });
  } catch (error) {
    await writeArtifactJson(artifactDir, "settings-fixture.error.json", {
      error: String(error),
    });
  }
}

export async function captureBrowserFailure(
  artifactDir: string,
  name: string,
): Promise<void> {
  await prepareArtifactDir(artifactDir);
  try {
    await fs.writeFile(
      path.join(artifactDir, `${name}.html`),
      await browser.getPageSource(),
      "utf8",
    );
    await browser.saveScreenshot(path.join(artifactDir, "screenshots", `${name}.png`));
  } catch (error) {
    await writeArtifactJson(artifactDir, `${name}.error.json`, {
      error: String(error),
    });
  }
}

export async function captureBrowserState(
  artifactDir: string,
  name: string,
  metadata: JsonRecord = {},
): Promise<void> {
  await prepareArtifactDir(artifactDir);
  try {
    await fs.writeFile(
      path.join(artifactDir, `${name}.html`),
      await browser.getPageSource(),
      "utf8",
    );
    await browser.saveScreenshot(path.join(artifactDir, "screenshots", `${name}.png`));
    await writeArtifactJson(artifactDir, `${name}.json`, metadata);
  } catch (error) {
    await writeArtifactJson(artifactDir, `${name}.error.json`, {
      error: String(error),
      metadata,
    });
    throw error;
  }
}