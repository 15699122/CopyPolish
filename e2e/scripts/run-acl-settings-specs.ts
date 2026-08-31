import { spawnSync } from "node:child_process";
import fs from "node:fs";
import net from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  createWindowsAclFixture,
  restoreWindowsAclFixture,
} from "../support/windows-acl.js";

const e2eDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const wdioCli = path.join(e2eDir, "node_modules", "@wdio", "cli", "bin", "wdio.js");
const useWebdriver = process.argv.includes("--webdriver");
const config = useWebdriver ? "wdio.webdriver.conf.ts" : "wdio.conf.ts";
const spec = path.join(e2eDir, "specs", "acl-settings.spec.ts");

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

if (process.platform !== "win32") {
  console.log("Skipping NTFS ACL E2E: native Windows is required.");
  process.exit(0);
}

const fixture = createWindowsAclFixture(e2eDir);
const artifactDir = path.join(
  e2eDir,
  "artifacts",
  useWebdriver ? "webdriver" : "embedded",
  `${Date.now()}-acl-settings`,
);
const port = useWebdriver ? await findFreePort() : undefined;

try {
  let exitCode = 0;
  const args = [wdioCli, "run", path.join(e2eDir, config), "--spec", spec];
  console.log(`\n=== Running ${useWebdriver ? "W3C" : "embedded"} Windows NTFS ACL settings spec ===`);
  const result = spawnSync(process.execPath, args, {
    cwd: e2eDir,
    env: {
      ...process.env,
      COPYPOLISH_E2E_KEEP_SETTINGS: "1",
      COPYPOLISH_E2E_SETTINGS_DIR: fixture.settingsDir,
      COPYPOLISH_E2E_ACL_SETTINGS_PATH: fixture.settingsPath,
      ...(useWebdriver
        ? {
            TAURI_WEBDRIVER_PORT: String(port),
            COPYPOLISH_E2E_ARTIFACT_DIR: artifactDir,
            VITE_COPYPOLISH_E2E_PROVIDER: "webdriver",
          }
        : {}),
    },
    stdio: "inherit",
  });

  if (result.error) throw result.error;
  exitCode = result.status ?? 1;
  process.exitCode = exitCode;
} finally {
  if (process.env.COPYPOLISH_E2E_KEEP_SETTINGS && fs.existsSync(fixture.settingsDir)) {
    fs.mkdirSync(path.join(artifactDir, "settings-fixture"), { recursive: true });
    fs.cpSync(fixture.settingsDir, path.join(artifactDir, "settings-fixture"), { recursive: true });
  }
  restoreWindowsAclFixture(fixture);
}