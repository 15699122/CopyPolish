import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const e2eDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const matrixPath = path.join(e2eDir, "artifacts", "gui-dpi-matrix", "matrix.json");
const matrix = JSON.parse(await fs.promises.readFile(matrixPath, "utf8")) as {
  schemaVersion?: number;
  entries?: Array<{ provider?: string; actualScale?: number; artifactDir?: string }>;
};
if (matrix.schemaVersion !== 1 || !Array.isArray(matrix.entries)) {
  throw new Error("GUI DPI matrix manifest is invalid");
}
const missing: string[] = [];
for (const scale of [100, 125, 150]) {
  for (const provider of ["embedded", "webdriver"]) {
    const entry = matrix.entries.find(
      (candidate) => candidate.provider === provider && candidate.actualScale === scale,
    );
    if (!entry?.artifactDir) {
      missing.push(`${provider}@${scale}%`);
      continue;
    }
    const artifactDir = path.resolve(e2eDir, entry.artifactDir);
    for (const relative of [
      "dpi-environment.json",
      "visual-states.json",
      "screenshots/main-normal.png",
      "screenshots/settings-light.png",
      "screenshots/settings-dark.png",
      "screenshots/main-narrow.png",
      "screenshots/settings-narrow.png",
    ]) {
      if (!fs.existsSync(path.join(artifactDir, relative))) {
        missing.push(`${provider}@${scale}%:${relative}`);
      }
    }
  }
}
if (missing.length > 0) {
  throw new Error(`GUI DPI matrix incomplete: ${missing.join(", ")}`);
}
console.log("GUI DPI matrix complete: embedded/webdriver at 100%, 125%, 150%");