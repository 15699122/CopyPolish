import path from "node:path";
import { fileURLToPath } from "node:url";
import fs from "node:fs/promises";
import { assertNoRepositorySettings } from "./settings.js";

export const e2eDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
export const rootDir = path.resolve(e2eDir, "..");

export async function waitForApp(): Promise<void> {
  try {
    await browser.waitUntil(
      async () => (await $("[data-testid=\"input-textarea\"]")).isExisting(),
      { timeout: 30_000, timeoutMsg: "真实 Tauri 主界面未启动" },
    );
  } catch (error) {
    const diagnostics = await browser.execute(() => ({
      href: window.location.href,
      readyState: document.readyState,
      body: document.body?.innerHTML ?? "",
      root: document.getElementById("root")?.innerHTML ?? null,
      scripts: Array.from(document.scripts).map((script) => ({
        src: script.src,
        async: script.async,
        defer: script.defer,
      })),
      tauri: Boolean((window as Window & { __TAURI__?: unknown }).__TAURI__),
      wdioTauri: Boolean((window as Window & { wdioTauri?: unknown }).wdioTauri),
    }));
    await fs.writeFile(
      path.join(e2eDir, "artifacts", "startup-diagnostics.json"),
      JSON.stringify(diagnostics, null, 2),
      "utf8",
    );
    await fs.writeFile(
      path.join(e2eDir, "artifacts", "startup-page-source.html"),
      await browser.getPageSource(),
      "utf8",
    );
    throw error;
  }
}

export async function assertAppDidNotPolluteRepository(): Promise<void> {
  await assertNoRepositorySettings(rootDir);
}

export async function formatText(inputText: string): Promise<string> {
  const input = await $("[data-testid=\"input-textarea\"]");
  const output = await $("[data-testid=\"output-text\"]");
  await input.setValue(inputText);
  let previousOutput = "";
  let stableReads = 0;
  await browser.waitUntil(
    async () => {
      const currentOutput = await output.getText();
      if (currentOutput === "") {
        previousOutput = "";
        stableReads = 0;
        return false;
      }
      stableReads = currentOutput === previousOutput ? stableReads + 1 : 0;
      previousOutput = currentOutput;
      return stableReads >= 1;
    },
    { timeout: 15_000, timeoutMsg: "真实格式化结果未稳定出现" },
  );
  return output.getText();
}