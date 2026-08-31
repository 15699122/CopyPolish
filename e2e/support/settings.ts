import fs from "node:fs/promises";
import path from "node:path";

export const SETTINGS_FILE_NAME = "rules.yaml";
export const BACKUP_FILE_NAME = "rules.yaml.bak";

export function settingsPath(dir: string): string {
  return path.join(dir, SETTINGS_FILE_NAME);
}

export function backupPath(dir: string): string {
  return path.join(dir, BACKUP_FILE_NAME);
}

export async function writeFile(dir: string, name: string, content: string): Promise<void> {
  await fs.writeFile(path.join(dir, name), content, "utf8");
}

export async function readSettings(dir: string): Promise<string> {
  return fs.readFile(settingsPath(dir), "utf8");
}

export async function removeSettingsDir(dir: string): Promise<void> {
  await fs.rm(dir, { recursive: true, force: true });
}

export async function assertNoRepositorySettings(rootDir: string): Promise<void> {
  for (const name of [SETTINGS_FILE_NAME, BACKUP_FILE_NAME]) {
    const filePath = path.join(rootDir, name);
    try {
      await fs.access(filePath);
    } catch (error) {
      if ((error as NodeJS.ErrnoException).code === "ENOENT") continue;
      throw error;
    }
    throw new Error(`E2E created repository settings file: ${filePath}`);
  }
}
