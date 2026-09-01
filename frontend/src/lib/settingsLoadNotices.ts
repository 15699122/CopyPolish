import type { SettingsLoadNotice } from "@/lib/tauri";

/** 设置加载提醒的唯一文案来源；主界面提示条与设置弹窗底部共用。 */
export const SETTINGS_LOAD_NOTICE_TEXT: Record<SettingsLoadNotice, string> = {
  legacy_settings_detected: "检测到旧版本设置文件，已迁移至 rules.yaml。",
  legacy_settings_corrupt: "检测到旧版本设置文件，但内容无法读取，已使用默认设置。",
  primary_settings_corrupt_recovered_from_backup: "设置文件损坏，已从 rules.yaml.bak 恢复。",
  primary_settings_corrupt_no_usable_backup: "设置文件损坏，且备份文件也无法读取，已使用默认设置。",
  backup_settings_corrupt: "备份文件损坏，当前 rules.yaml 仍可正常使用。",
  using_app_data_fallback:
    "程序目录不可写，设置已改用应用数据目录保存（设置窗口底部可见实际路径）。",
};

export function settingsLoadNoticeText(notice: SettingsLoadNotice): string {
  return SETTINGS_LOAD_NOTICE_TEXT[notice];
}

/** 需要以 assert 语义（强制刷新）呈现的提醒。 */
export function isSettingsLoadNoticeAlert(notice: SettingsLoadNotice): boolean {
  return notice === "primary_settings_corrupt_no_usable_backup" || notice === "legacy_settings_corrupt";
}