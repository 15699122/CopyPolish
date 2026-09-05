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
  await setTextInput(inputText);
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

export async function setTextInput(value: string): Promise<void> {
  await browser.execute((nextValue) => {
    const input = document.querySelector<HTMLTextAreaElement>("[data-testid=\"input-textarea\"]");
    if (!input) throw new Error("输入框不存在");
    const setter = Object.getOwnPropertyDescriptor(HTMLTextAreaElement.prototype, "value")?.set;
    setter?.call(input, nextValue);
    input.dispatchEvent(new Event("input", { bubbles: true }));
    input.dispatchEvent(new Event("change", { bubbles: true }));
  }, value);
  await browser.waitUntil(
    async () => await (await $("[data-testid=\"input-textarea\"]")).getValue() === value,
    { timeout: 10_000, timeoutMsg: "输入框未接收目标文本" },
  );
}

export async function getE2EDiagnostics(): Promise<Record<string, unknown>> {
  return await browser.execute(() => ({
    ...((window as Window & { __COPYPOLISH_E2E__?: Record<string, unknown> }).__COPYPOLISH_E2E__ ?? {}),
    inputValue: document.querySelector<HTMLTextAreaElement>("[data-testid=\"input-textarea\"]")?.value ?? null,
    outputText: document.querySelector<HTMLElement>("[data-testid=\"output-text\"]")?.innerText ?? null,
    settingsStatus: document.querySelector<HTMLElement>("[data-testid=\"settings-status\"]")?.innerText ?? null,
    conversion: document.querySelector<HTMLSelectElement>("[data-testid=\"conversion-select\"]")?.value ?? null,
    replacementFrom: document.querySelector<HTMLInputElement>("[data-testid=\"replacement-from-0\"]")?.value ?? null,
    replacementTo: document.querySelector<HTMLInputElement>("[data-testid=\"replacement-to-0\"]")?.value ?? null,
  }));
}