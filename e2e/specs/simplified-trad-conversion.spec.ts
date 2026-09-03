import { assertAppDidNotPolluteRepository, waitForApp } from "../support/app.js";
import { readSettings } from "../support/settings.js";

async function selectConversion(value: "t2s" | "s2t"): Promise<void> {
  await browser.execute((nextValue) => {
    const select = document.querySelector<HTMLSelectElement>("[data-testid=\"conversion-select\"]");
    if (!select) throw new Error("简繁转换选择器不存在");
    const setter = Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype, "value")?.set;
    setter?.call(select, nextValue);
    select.dispatchEvent(new Event("change", { bubbles: true }));
  }, value);
  await browser.waitUntil(
    async () => await (await $("[data-testid=\"conversion-select\"]")).getValue() === value,
    { timeout: 10_000, timeoutMsg: `简繁转换选择未更新为 ${value}` },
  );
}

async function waitForSavedConversion(value: "t2s" | "s2t", previousSequence: number): Promise<void> {
  await browser.waitUntil(
    async () => {
      const diagnostics = await browser.execute((): Record<string, unknown> => ({
        ...((window as Window & { __COPYPOLISH_E2E__?: Record<string, unknown> }).__COPYPOLISH_E2E__ ?? {}),
        settingsStatus: document.querySelector<HTMLElement>("[data-testid=\"settings-status\"]")?.innerText ?? null,
      }));
      const saved = diagnostics.lastSettingsSave;
      return diagnostics.settingsStatus === "设置已保存"
        && Number(diagnostics.settingsSaveSequence ?? 0) > previousSequence
        && typeof saved === "object"
        && saved !== null
        && (saved as { conversion?: unknown }).conversion === value;
    },
    { timeout: 10_000, timeoutMsg: `简繁转换设置未保存为 ${value}` },
  );
}

async function waitForSavedSequence(previousSequence: number): Promise<void> {
  await browser.waitUntil(
    async () => {
      const diagnostics = await browser.execute((): Record<string, unknown> => ({
        ...((window as Window & { __COPYPOLISH_E2E__?: Record<string, unknown> }).__COPYPOLISH_E2E__ ?? {}),
        settingsStatus: document.querySelector<HTMLElement>("[data-testid=\"settings-status\"]")?.innerText ?? null,
      }));
      return diagnostics.settingsStatus === "设置已保存"
        && Number(diagnostics.settingsSaveSequence ?? 0) > previousSequence;
    },
    { timeout: 10_000, timeoutMsg: "设置保存序号未更新" },
  );
}

async function prepareConversion(value: "t2s" | "s2t"): Promise<void> {
  await $("[data-testid=\"open-settings\"]").click();
  const selectNone = await $("[data-testid=\"select-none\"]");
  await selectNone.waitForDisplayed({ timeout: 10_000 });
  const beforeRuleSave = Number((await browser.execute(() =>
    (window as Window & { __COPYPOLISH_E2E__?: Record<string, unknown> }).__COPYPOLISH_E2E__?.settingsSaveSequence ?? 0,
  ))) || 0;
  await selectNone.click();
  await waitForSavedSequence(beforeRuleSave);
  const beforeConversionSave = Number((await browser.execute(() =>
    (window as Window & { __COPYPOLISH_E2E__?: Record<string, unknown> }).__COPYPOLISH_E2E__?.settingsSaveSequence ?? 0,
  ))) || 0;
  await selectConversion(value);
  await waitForSavedConversion(value, beforeConversionSave);
  await $("[data-testid=\"settings-done\"]").click();
}

async function expectConvertedOutput(inputValue: string, expectedValue: string): Promise<void> {
  const input = await $("[data-testid=\"input-textarea\"]");
  const output = await $("[data-testid=\"output-text\"]");
  await input.setValue(inputValue);
  await browser.waitUntil(
    async () => (await output.getText()).includes(expectedValue),
    { timeout: 15_000, timeoutMsg: `feature 构建未生成预期转换结果：${expectedValue}` },
  );
  expect(await output.getText()).toContain(expectedValue);
}

describe("CopyPolish simplified-trad-conversion feature GUI", () => {
  before(async () => {
    await waitForApp();
    const diagnostics = await browser.execute(() =>
      (window as Window & { __COPYPOLISH_E2E__?: Record<string, unknown> }).__COPYPOLISH_E2E__ ?? {},
    );
    expect(diagnostics.buildCapabilities).toEqual({ simplifiedTradConversion: true });
  });

  after(async () => {
    await assertAppDidNotPolluteRepository();
  });

  it("GUI 选择 s2t 后通过真实 Rust IPC 执行简体转繁体", async () => {
    await prepareConversion("s2t");

    const settingsDir = process.env.COPYPOLISH_E2E_SETTINGS_DIR;
    expect(settingsDir).toBeTruthy();
    const settings = await readSettings(settingsDir!);
    expect(settings).toContain("conversion: s2t");

    await expectConvertedOutput("设计软件与打印", "設計軟件與打印");
  });

  it("GUI 选择 t2s 后通过真实 Rust IPC 执行繁体转简体", async () => {
    await prepareConversion("t2s");

    const settingsDir = process.env.COPYPOLISH_E2E_SETTINGS_DIR;
    expect(settingsDir).toBeTruthy();
    const settings = await readSettings(settingsDir!);
    expect(settings).toContain("conversion: t2s");

    await expectConvertedOutput("後設資料與說明", "后设资料与说明");
  });
});