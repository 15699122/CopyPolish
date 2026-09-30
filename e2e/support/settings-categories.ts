// e2e/support/settings-categories.ts
// =============================================================================
// 设置弹窗的分类导航辅助。
//
// 设置窗口按任务分为六个分类，非当前分类的内容不会挂载到 DOM。E2E 在访问
// 规则以外的控件（替换、转换、主题、快捷键、隐私等）前，必须先切换到对应
// 分类，否则元素不存在或不可见。
// =============================================================================

export const SETTINGS_CATEGORY_IDS = [
  "rules",
  "transform",
  "output",
  "appearance",
  "shortcuts",
  "privacy",
] as const;

export type SettingsCategoryId = (typeof SETTINGS_CATEGORY_IDS)[number];

/**
 * 打开设置弹窗并切换到指定分类。
 *
 * 若弹窗已打开则只切换分类，避免重复点击触发器导致状态不一致。调用完成后
 * 会等待目标分类按钮处于选中状态（`aria-current="true"`），并等待该分类的
 * 滚动区 `aria-label` 与分类名一致，确保内容已挂载。
 */
export async function openSettingsCategory(category: SettingsCategoryId): Promise<void> {
  const nav = await $("[data-testid=\"settings-category-nav\"]");
  if (!(await nav.isExisting())) {
    await $("[data-testid=\"open-settings\"]").click();
    await browser.waitUntil(
      async () => await (await $("[data-testid=\"settings-category-nav\"]")).isExisting(),
      { timeout: 10_000, timeoutMsg: "设置弹窗未完成渲染" },
    );
  }

  const button = await $(`[data-testid="settings-category-${category}"]`);
  await button.waitForDisplayed({ timeout: 10_000 });
  await button.click();
  await browser.waitUntil(
    async () => (await button.getAttribute("aria-current")) === "true",
    { timeout: 10_000, timeoutMsg: `设置分类未切换到 ${category}` },
  );
}

/** 在已打开的设置弹窗内切换分类，不负责打开弹窗。 */
export async function switchSettingsCategory(category: SettingsCategoryId): Promise<void> {
  const button = await $(`[data-testid="settings-category-${category}"]`);
  await button.waitForDisplayed({ timeout: 10_000 });
  await button.click();
  await browser.waitUntil(
    async () => (await button.getAttribute("aria-current")) === "true",
    { timeout: 10_000, timeoutMsg: `设置分类未切换到 ${category}` },
  );
}

/** 关闭设置弹窗并等待其卸载。 */
export async function closeSettings(): Promise<void> {
  const done = await $("[data-testid=\"settings-done\"]");
  if (!(await done.isExisting())) return;
  await done.click();
  await browser.waitUntil(
    async () => !(await $("[data-testid=\"settings-category-nav\"]")).isExisting(),
    { timeout: 10_000, timeoutMsg: "设置弹窗未关闭" },
  );
}