import fs from "node:fs";
import path from "node:path";

export type SettingsFixture =
  | "primary-corrupt-backup-valid"
  | "primary-corrupt-no-backup"
  | "primary-and-backup-corrupt";

const VALID_BACKUP = `enabled:
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

const CORRUPT_SETTINGS = "enabled: [this is not valid YAML\n";

export const SETTINGS_FIXTURES: readonly SettingsFixture[] = [
  "primary-corrupt-backup-valid",
  "primary-corrupt-no-backup",
  "primary-and-backup-corrupt",
];

export function prepareSettingsFixture(dir: string, fixture: SettingsFixture | undefined): void {
  if (!fixture) return;

  fs.mkdirSync(dir, { recursive: true });
  fs.writeFileSync(path.join(dir, "rules.yaml"), CORRUPT_SETTINGS, "utf8");

  if (fixture === "primary-corrupt-backup-valid") {
    fs.writeFileSync(path.join(dir, "rules.yaml.bak"), VALID_BACKUP, "utf8");
  } else if (fixture === "primary-and-backup-corrupt") {
    fs.writeFileSync(path.join(dir, "rules.yaml.bak"), CORRUPT_SETTINGS, "utf8");
  }
}

export function expectedFixtureNotice(fixture: SettingsFixture): string {
  return fixture === "primary-corrupt-backup-valid"
    ? "设置文件损坏，已从 rules.yaml.bak 恢复。"
    : "设置文件损坏，且备份文件也无法读取，已使用默认设置。";
}