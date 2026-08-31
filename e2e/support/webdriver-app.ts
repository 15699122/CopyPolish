import { spawn, type ChildProcess } from "node:child_process";
import fs from "node:fs";
import fsPromises from "node:fs/promises";
import http from "node:http";
import path from "node:path";

export type WebDriverApp = {
  child: ChildProcess;
  port: number;
  pid: number;
  artifactDir: string;
};

const rootDir = path.resolve(import.meta.dirname, "../..");
const e2eDir = path.join(rootDir, "e2e");
const binaryName = process.platform === "win32"
  ? "chinese-copywriting-formatter.exe"
  : "chinese-copywriting-formatter";
const binaryPath = path.join(rootDir, "src-tauri", "target", "debug", binaryName);

function statusUrl(port: number): string {
  return `http://127.0.0.1:${port}/status`;
}

async function waitForStatus(port: number, child: ChildProcess, timeoutMs = 30_000): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  let lastError = "not attempted";

  while (Date.now() < deadline) {
    if (child.exitCode !== null) {
      throw new Error(`WebDriver application exited with code ${child.exitCode}; last status error: ${lastError}`);
    }

    try {
      const response = await fetch(statusUrl(port));
      if (response.ok) return;
      lastError = `HTTP ${response.status}`;
    } catch (error) {
      lastError = error instanceof Error ? error.message : String(error);
    }

    await new Promise((resolve) => setTimeout(resolve, 250));
  }

  throw new Error(`Timed out waiting for WebDriver status at ${statusUrl(port)}; last error: ${lastError}`);
}

export async function startWebDriverApp(port: number): Promise<WebDriverApp> {
  const settingsDir = process.env.COPYPOLISH_E2E_SETTINGS_DIR;
  if (!settingsDir) throw new Error("COPYPOLISH_E2E_SETTINGS_DIR is required");

  const artifactDir = process.env.COPYPOLISH_E2E_ARTIFACT_DIR
    ?? path.join(e2eDir, "artifacts", "webdriver", `${Date.now()}-${port}`);
  await fsPromises.mkdir(artifactDir, { recursive: true });

  const stdoutFd = fs.openSync(path.join(artifactDir, "backend.stdout.log"), "w");
  const stderrFd = fs.openSync(path.join(artifactDir, "backend.stderr.log"), "w");
  const child = spawn(binaryPath, [], {
    cwd: rootDir,
    env: {
      ...process.env,
      TAURI_WEBDRIVER_PORT: String(port),
      COPYPOLISH_E2E_SETTINGS_DIR: settingsDir,
      VITE_COPYPOLISH_E2E: "true",
      GDK_BACKEND: process.env.GDK_BACKEND ?? "x11",
      LIBGL_ALWAYS_SOFTWARE: process.env.LIBGL_ALWAYS_SOFTWARE ?? "1",
      MESA_LOADER_DRIVER_OVERRIDE: process.env.MESA_LOADER_DRIVER_OVERRIDE ?? "llvmpipe",
    },
    stdio: ["ignore", stdoutFd, stderrFd],
    windowsHide: true,
  });

  if (!child.pid) {
    fs.closeSync(stdoutFd);
    fs.closeSync(stderrFd);
    throw new Error("Failed to spawn WebDriver application");
  }
  child.once("exit", () => {
    fs.closeSync(stdoutFd);
    fs.closeSync(stderrFd);
  });
  const app: WebDriverApp = { child, port, pid: child.pid, artifactDir };
  await fsPromises.writeFile(
    path.join(artifactDir, "manifest.json"),
    `${JSON.stringify({
      provider: "tauri-plugin-webdriver",
      port,
      pid: child.pid,
      binaryPath,
      platform: process.platform,
      node: process.version,
      commit: process.env.CI_COMMIT_SHA ?? "local",
      settingsDir,
    }, null, 2)}\n`,
    "utf8",
  );

  try {
    await waitForStatus(port, child);
  } catch (error) {
    await stopWebDriverApp(app);
    throw error;
  }

  return app;
}

function requestShutdown(port: number): Promise<void> {
  return new Promise((resolve) => {
    const request = http.request({ hostname: "127.0.0.1", port, path: "/status", method: "GET" }, (response) => {
      response.resume();
      response.on("end", resolve);
    });
    request.on("error", () => resolve());
    request.setTimeout(500, () => {
      request.destroy();
      resolve();
    });
    request.end();
  });
}

export async function stopWebDriverApp(app: WebDriverApp): Promise<void> {
  await requestShutdown(app.port);

  if (app.child.exitCode === null) {
    app.child.kill("SIGTERM");
    await new Promise<void>((resolve) => {
      const timer = setTimeout(() => {
        if (app.child.exitCode === null) app.child.kill("SIGKILL");
        resolve();
      }, 3_000);
      app.child.once("exit", () => {
        clearTimeout(timer);
        resolve();
      });
    });
  }

  await fsPromises.writeFile(
    path.join(app.artifactDir, "exit.json"),
    `${JSON.stringify({ code: app.child.exitCode, signal: app.child.signalCode }, null, 2)}\n`,
    "utf8",
  );
}