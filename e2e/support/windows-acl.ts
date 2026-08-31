import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";

const VALID_SETTINGS = `enabled:
  - spacing.cjk-latin
  - spacing.cjk-number
last_input: ""
theme: system
font: system
editor_font_size: normal
ui_scale: normal
shortcuts:
  enabled: true
  bindings:
    format_now: CtrlOrCmd+Enter
    copy_output: CtrlOrCmd+Shift+KeyC
    open_settings: CtrlOrCmd+Comma
`;

function runWindowsCommand(command: string, args: string[]): void {
  execFileSync(command, args, { stdio: "pipe", windowsHide: true });
}

function currentWindowsUser(): string {
  return execFileSync("whoami.exe", [], { encoding: "utf8", windowsHide: true }).trim();
}

export type WindowsAclFixture = {
  settingsDir: string;
  settingsPath: string;
  user: string;
};

export function createWindowsAclFixture(rootDir: string): WindowsAclFixture {
  if (process.platform !== "win32") {
    throw new Error("Windows NTFS ACL fixtures require a native Windows runner");
  }

  const settingsDir = fs.mkdtempSync(path.join(rootDir, "settings-acl-"));
  const settingsPath = path.join(settingsDir, "rules.yaml");
  const user = currentWindowsUser();

  try {
    fs.writeFileSync(settingsPath, VALID_SETTINGS, "utf8");
    runWindowsCommand("icacls.exe", [settingsDir, "/inheritance:r"]);
    runWindowsCommand("icacls.exe", [settingsDir, "/deny", `${user}:(OI)(CI)(W)`]);
  } catch (error) {
    fs.rmSync(settingsDir, { recursive: true, force: true });
    throw error;
  }

  return { settingsDir, settingsPath, user };
}

export function restoreWindowsAclFixture(fixture: WindowsAclFixture): void {
  if (process.platform !== "win32") return;

  try {
    runWindowsCommand("icacls.exe", [fixture.settingsDir, "/remove:d", fixture.user]);
  } finally {
    try {
      runWindowsCommand("icacls.exe", [fixture.settingsDir, "/inheritance:e"]);
    } finally {
      fs.rmSync(fixture.settingsDir, { recursive: true, force: true });
    }
  }
}