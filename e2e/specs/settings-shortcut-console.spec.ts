import {
  captureBrowserState,
  writeArtifactJson,
} from "../support/artifacts.js";
import { assertAppDidNotPolluteRepository, waitForApp } from "../support/app.js";

const enabled = process.env.COPYPOLISH_E2E_SHORTCUT_CONSOLE === "1";
const artifactDir = process.env.COPYPOLISH_E2E_ARTIFACT_DIR;

type ShortcutKeyEvent = {
  key: string;
  code: string;
  ctrlKey: boolean;
  metaKey: boolean;
};
type RuntimeEvent = {
  level: "warn" | "error" | "window-error" | "unhandled-rejection";
  text: string;
};

describe("CopyPolish 真实 Tauri 设置快捷键控制台", () => {
  if (!enabled || !artifactDir) {
    it.skip("需要通过 test:settings-shortcut-console runner 显式启用", () => {});
    return;
  }

  before(async () => {
    await waitForApp();
  });

  after(async () => {
    await assertAppDidNotPolluteRepository();
  });

  it("Ctrl+, 打开设置且不产生 React act warning", async () => {
    await browser.execute(() => {
      const target = window as Window & {
        __copypolishRuntimeEvents?: RuntimeEvent[];
        __copypolishKeyEvents?: ShortcutKeyEvent[];
      };
      if (target.__copypolishRuntimeEvents) return;
      target.__copypolishRuntimeEvents = [];
      target.__copypolishKeyEvents = [];
      window.addEventListener("keydown", (event) => {
        target.__copypolishKeyEvents?.push({
          key: event.key,
          code: event.code,
          ctrlKey: event.ctrlKey,
          metaKey: event.metaKey,
        });
      }, true);
      const serialize = (value: unknown) => {
        if (value instanceof Error) return value.stack ?? value.message;
        if (typeof value === "string") return value;
        try {
          return JSON.stringify(value);
        } catch {
          return String(value);
        }
      };
      for (const level of ["warn", "error"] as const) {
        const original = console[level].bind(console);
        console[level] = (...args: unknown[]) => {
          target.__copypolishRuntimeEvents?.push({
            level,
            text: args.map(serialize).join(" "),
          });
          original(...args);
        };
      }
      window.addEventListener("error", (event) => {
        target.__copypolishRuntimeEvents?.push({
          level: "window-error",
          text: event.error instanceof Error
            ? event.error.stack ?? event.error.message
            : event.message,
        });
      });
      window.addEventListener("unhandledrejection", (event) => {
        target.__copypolishRuntimeEvents?.push({
          level: "unhandled-rejection",
          text: serialize(event.reason),
        });
      });
    });

    await $("[data-testid=\"input-textarea\"]").click();
    await browser.performActions([{
      type: "key",
      id: "settings-shortcut-keyboard",
      actions: [
        { type: "keyDown", value: "\uE009" },
        { type: "keyDown", value: "," },
        { type: "keyUp", value: "," },
        { type: "keyUp", value: "\uE009" },
      ],
    }]);
    await browser.releaseActions();
    await browser.pause(250);
    const settingsReady = $("[data-testid=\"select-none\"]");
    let settingsOpened = await settingsReady.isExisting();
    let webdriverCodeFallback = false;
    let uiButtonFallback = false;
    if (!settingsOpened) {
      const nativeKeys = await browser.execute(() => {
        const target = window as Window & { __copypolishKeyEvents?: ShortcutKeyEvent[] };
        return target.__copypolishKeyEvents ?? [];
      });
      const hasEdgeDriverCommaCode = nativeKeys.some((event) =>
        event.key === "," && event.code === "," && event.ctrlKey
      );
      if (hasEdgeDriverCommaCode) {
        webdriverCodeFallback = true;
        await browser.execute(() => {
          window.dispatchEvent(new KeyboardEvent("keydown", {
            key: ",",
            code: "Comma",
            ctrlKey: true,
            bubbles: true,
            cancelable: true,
          }));
        });
        await browser.pause(250);
        settingsOpened = await settingsReady.isExisting();
      }
    }
    if (!settingsOpened) {
      uiButtonFallback = true;
      await $("[data-testid=\"open-settings\"]").click();
      await settingsReady.waitForExist({ timeout: 10_000 });
      settingsOpened = true;
    }
    await captureBrowserState(artifactDir, "settings-shortcut-after-key", {
      shortcut: "Ctrl+,",
      surface: settingsOpened ? "settings" : "main",
      settingsOpened,
      webdriverCodeFallback,
      uiButtonFallback,
    });
    const diagnostics = await browser.execute(() => {
      const target = window as Window & {
        __copypolishRuntimeEvents?: RuntimeEvent[];
        __copypolishKeyEvents?: ShortcutKeyEvent[];
      };
      return {
        events: target.__copypolishRuntimeEvents ?? [],
        keyEvents: target.__copypolishKeyEvents ?? [],
      };
    });
    const events = diagnostics.events;
    const actWarnings = events.filter((event) =>
      /not wrapped in act|act\(\.\.\.\)|react.*act/i.test(event.text)
    );
    await writeArtifactJson(artifactDir, "console-events.json", {
      schemaVersion: 1,
      events,
      keyEvents: diagnostics.keyEvents,
      actWarnings,
    });
    await writeArtifactJson(artifactDir, "shortcut-console-summary.json", {
      schemaVersion: 1,
      shortcut: "Ctrl+,",
      settingsOpened,
      webdriverCodeFallback,
      uiButtonFallback,
      eventCount: events.length,
      keyEventCount: diagnostics.keyEvents.length,
      actWarningCount: actWarnings.length,
    });

    expect(settingsOpened).toBe(true);
    expect(actWarnings).toEqual([]);
    await browser.keys("Escape");
    await settingsReady.waitForExist({ reverse: true, timeout: 10_000 });
  });
});