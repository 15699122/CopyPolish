// user_settings.rs
// =============================================================================
// 用户设置持久化：保存在「可执行文件所在目录」下的 rules.yaml（YAML 格式）。
// Windows 便携版运行后在 exe 相同目录生成 rules.yaml；目录不可写时向前端返回
// 带路径的明确错误，便于用户把便携版放到可写目录。
//
// 迁移：若 rules.yaml 不存在但同目录存在旧版 ccw-formatter-settings.json，
// 自动读取旧 JSON 并转换写入新的 rules.yaml。
//
// 注意：单元测试一律使用系统临时目录中的随机文件，绝不写仓库内文件，
// 避免测试之间 / 测试与真实设置之间的写覆盖。
// =============================================================================

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

/// 用户设置文件名（exe 同目录，YAML 格式）。
pub const SETTINGS_FILE_NAME: &str = "rules.yaml";
/// 设置备份文件名；主文件损坏时作为最后一次有效保存的恢复来源。
pub const SETTINGS_BACKUP_FILE_NAME: &str = "rules.yaml.bak";

/// 由设置文件路径派生备份路径：在文件名后追加 `.bak`。
/// 生产设置文件固定为 `rules.yaml`，派生结果即 `rules.yaml.bak`；
/// 测试使用随机临时文件名时各自得到唯一备份路径，避免并行测试共享
/// 同一备份文件产生竞态。
pub(crate) fn backup_path_for(settings_path: &Path) -> PathBuf {
    let mut name = settings_path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(".bak");
    settings_path.with_file_name(name)
}
/// 旧版设置文件名（JSON，仅用于一次性迁移读取）。
pub const LEGACY_SETTINGS_FILE_NAME: &str = "ccw-formatter-settings.json";

/// 主题模式：跟随系统 / 浅色 / 深色。
/// `#[serde(default)]` 确保旧版设置文件（无 theme 字段）可反序列化，
/// 默认回退为 `System`。
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    #[default]
    System,
    Light,
    Dark,
}

/// 界面字体预设；实际字体栈由前端根据 key 应用，未安装字体自动回退。
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum FontFamily {
    #[default]
    System,
    MicrosoftYahei,
    Pingfang,
    NotoSansCjk,
    Simsun,
    Simhei,
}

/// 主界面编辑器字号预设。
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum EditorFontSize {
    Small,
    #[default]
    Normal,
    Large,
    XLarge,
}

/// 主界面整体缩放预设。
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum UiScale {
    Compact,
    Small,
    #[default]
    Normal,
    Large,
    XLarge,
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ThemeMode::System => write!(f, "system"),
            ThemeMode::Light => write!(f, "light"),
            ThemeMode::Dark => write!(f, "dark"),
        }
    }
}

/// 快捷键设置：总开关与各动作的绑定。
/// `#[serde(default)]` 确保旧版设置文件（无 shortcuts 字段）可反序列化，
/// 默认启用并使用内置默认组合键，行为与旧版本一致。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ShortcutSettings {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_shortcut_bindings")]
    pub bindings: std::collections::BTreeMap<String, String>,
}

fn default_true() -> bool {
    true
}

/// 与前端 `frontend/src/lib/shortcuts.ts` 的 DEFAULT_SHORTCUT_BINDINGS 保持一致；
/// 组合键使用语义修饰键 CtrlOrCmd + KeyboardEvent.code 序列化。
pub fn default_shortcut_bindings() -> std::collections::BTreeMap<String, String> {
    std::collections::BTreeMap::from([
        ("format_now".to_string(), "CtrlOrCmd+Enter".to_string()),
        (
            "copy_output".to_string(),
            "CtrlOrCmd+Shift+KeyC".to_string(),
        ),
        ("open_settings".to_string(), "CtrlOrCmd+Comma".to_string()),
    ])
}

impl Default for ShortcutSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            bindings: default_shortcut_bindings(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq, Eq)]
pub struct UserSettings {
    #[serde(default)]
    pub enabled: Vec<String>,
    #[serde(default)]
    pub last_input: String,
    #[serde(default)]
    pub theme: ThemeMode,
    #[serde(default)]
    pub font: FontFamily,
    #[serde(default)]
    pub editor_font_size: EditorFontSize,
    #[serde(default)]
    pub ui_scale: UiScale,
    #[serde(default)]
    pub shortcuts: ShortcutSettings,
}

/// 设置加载提醒类型。
#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SettingsLoadNotice {
    LegacySettingsDetected,
    LegacySettingsCorrupt,
    PrimarySettingsCorruptRecoveredFromBackup,
    PrimarySettingsCorruptNoUsableBackup,
    BackupSettingsCorrupt,
}

/// 设置加载结果及加载阶段产生的提醒。
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct LoadedUserSettings {
    pub settings: UserSettings,
    pub notices: Vec<SettingsLoadNotice>,
}

/// 设置保存目录：优先使用当前可执行文件所在目录，失败时回退当前工作目录。
fn settings_dir() -> PathBuf {
    #[cfg(feature = "e2e")]
    if let Ok(dir) = std::env::var("COPYPOLISH_E2E_SETTINGS_DIR") {
        if !dir.is_empty() {
            return PathBuf::from(dir);
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.to_path_buf();
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn settings_path() -> PathBuf {
    settings_dir().join(SETTINGS_FILE_NAME)
}

/// 从指定路径读取 YAML 设置；文件缺失或解析失败时返回 None（调用方自行回落默认值）。
pub fn load_from(path: &Path) -> Option<UserSettings> {
    let text = fs::read_to_string(path).ok()?;
    serde_yaml::from_str(&text).ok()
}

pub fn save_to(path: &Path, settings: &UserSettings) -> Result<(), String> {
    let yaml = serde_yaml::to_string(settings)
        .map_err(|e| format!("serialize settings for {}: {e}", path.display()))?;
    let dir = path
        .parent()
        .ok_or_else(|| format!("settings path has no parent directory: {}", path.display()))?;

    if !dir.exists() {
        return Err(format!(
            "settings directory does not exist: {}; target settings file is {}",
            dir.display(),
            path.display()
        ));
    }
    if !dir.is_dir() {
        return Err(format!(
            "settings parent path is not a directory: {}",
            dir.display()
        ));
    }

    let tmp_path = path.with_extension("yaml.tmp");
    let mut file = fs::File::create(&tmp_path).map_err(|e| {
        format!(
            "create temporary settings file {} failed; target settings file is {}; directory is {}: {e}",
            tmp_path.display(),
            path.display(),
            dir.display()
        )
    })?;
    file.write_all(yaml.as_bytes()).map_err(|e| {
        format!(
            "write temporary settings file {} failed; target settings file is {}: {e}",
            tmp_path.display(),
            path.display()
        )
    })?;
    file.sync_all().map_err(|e| {
        format!(
            "flush temporary settings file {} failed; target settings file is {}: {e}",
            tmp_path.display(),
            path.display()
        )
    })?;
    drop(file);

    // 先保留当前有效文件，再替换主文件。若备份轮换失败，主文件仍保持不变。
    let mut backup_moved = false;
    let backup_path = backup_path_for(path);
    if path.exists() {
        if backup_path.exists() {
            fs::remove_file(&backup_path).map_err(|e| {
                format!(
                    "remove previous settings backup {} failed; target settings file is {}: {e}",
                    backup_path.display(),
                    path.display()
                )
            })?;
        }
        fs::rename(path, &backup_path).map_err(|e| {
            format!(
                "backup settings file {} to {} failed: {e}",
                path.display(),
                backup_path.display()
            )
        })?;
        backup_moved = true;
    }

    if let Err(error) = fs::rename(&tmp_path, path) {
        let _ = fs::remove_file(&tmp_path);
        if backup_moved && !path.exists() {
            let _ = fs::rename(&backup_path, path);
        }
        return Err(format!(
            "replace settings file {} with temporary file {} failed; directory is {}: {error}",
            path.display(),
            tmp_path.display(),
            dir.display()
        ));
    }
    Ok(())
}

/// 读取指定目录下的设置，并在主文件损坏时尝试使用备份。
pub fn load_from_dir_with_status(dir: &Path) -> Option<LoadedUserSettings> {
    let path = dir.join(SETTINGS_FILE_NAME);
    let backup_path = dir.join(SETTINGS_BACKUP_FILE_NAME);
    let legacy = dir.join(LEGACY_SETTINGS_FILE_NAME);
    let primary_exists = path.exists();
    let backup_exists = backup_path.exists();

    if let Some(settings) = load_from(&path) {
        let mut notices = Vec::new();
        if backup_exists && load_from(&backup_path).is_none() {
            notices.push(SettingsLoadNotice::BackupSettingsCorrupt);
        }
        return Some(LoadedUserSettings { settings, notices });
    }

    if let Some(settings) = load_from(&backup_path) {
        return Some(LoadedUserSettings {
            settings,
            notices: vec![SettingsLoadNotice::PrimarySettingsCorruptRecoveredFromBackup],
        });
    }

    if legacy.exists() {
        if let Ok(json) = fs::read_to_string(&legacy) {
            if let Ok(settings) = serde_json::from_str::<UserSettings>(&json) {
                // 迁移成功与否不影响本次返回；失败则下次再试。
                let _ = save_to(&path, &settings);
                return Some(LoadedUserSettings {
                    settings,
                    notices: vec![SettingsLoadNotice::LegacySettingsDetected],
                });
            }
        }
        return Some(LoadedUserSettings {
            settings: UserSettings::default(),
            notices: vec![SettingsLoadNotice::LegacySettingsCorrupt],
        });
    }

    if primary_exists || backup_exists {
        let mut notices = vec![SettingsLoadNotice::PrimarySettingsCorruptNoUsableBackup];
        if backup_exists {
            notices.push(SettingsLoadNotice::BackupSettingsCorrupt);
        }
        return Some(LoadedUserSettings {
            settings: UserSettings::default(),
            notices,
        });
    }
    None
}

/// 读取指定目录下的用户设置（含旧版 JSON 迁移逻辑），供测试注入目录。
#[cfg(test)]
pub fn load_from_dir(dir: &Path) -> Option<UserSettings> {
    load_from_dir_with_status(dir).map(|loaded| loaded.settings)
}

pub fn load_with_status() -> Option<LoadedUserSettings> {
    load_from_dir_with_status(&settings_dir())
}

pub fn save(settings: &UserSettings) -> Result<(), String> {
    save_to(&settings_path(), settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 系统临时目录内的唯一文件路径（进程 ID + 计数器），避免并行测试写覆盖。
    fn temp_settings_file(tag: &str) -> PathBuf {
        static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let pid = std::process::id();
        let mut path = std::env::temp_dir();
        path.push(format!("ccw-user-settings-test-{pid}-{n}-{tag}.yaml"));
        let _ = fs::remove_file(&path);
        path
    }

    #[test]
    fn missing_file_returns_none() {
        let path = temp_settings_file("missing");
        assert_eq!(load_from(&path), None);
    }

    #[test]
    fn roundtrip_preserves_fields() {
        let path = temp_settings_file("roundtrip");
        let settings = UserSettings {
            enabled: vec!["中英文之间需要增加空格".to_string()],
            last_input: "在LeanCloud上".to_string(),
            theme: ThemeMode::Dark,
            font: FontFamily::Pingfang,
            editor_font_size: EditorFontSize::Large,
            ui_scale: UiScale::Small,
            shortcuts: ShortcutSettings {
                enabled: false,
                bindings: default_shortcut_bindings(),
            },
        };
        save_to(&path, &settings).expect("save should succeed");
        assert_eq!(load_from(&path), Some(settings));
        // 保存格式必须是 YAML（含键名 enabled / last_input / theme）。
        let raw = fs::read_to_string(&path).expect("settings file must be readable");
        assert!(raw.contains("enabled"));
        assert!(raw.contains("last_input"));
        assert!(raw.contains("theme"));
        assert!(raw.contains("editor_font_size"));
        assert!(raw.contains("ui_scale"));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(backup_path_for(&path));
    }

    #[test]
    fn second_save_creates_backup_of_previous_settings() {
        let path = temp_settings_file("backup");
        let backup = backup_path_for(&path);
        let first = UserSettings {
            last_input: "第一次".to_string(),
            ..UserSettings::default()
        };
        let second = UserSettings {
            last_input: "第二次".to_string(),
            ..UserSettings::default()
        };

        save_to(&path, &first).expect("first save should succeed");
        save_to(&path, &second).expect("second save should succeed");

        assert_eq!(load_from(&path), Some(second));
        assert_eq!(load_from(&backup), Some(first));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&backup);
    }

    #[test]
    fn corrupt_primary_recovers_from_backup_with_status() {
        let dir = std::env::temp_dir().join(format!(
            "ccw-settings-recover-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let path = dir.join(SETTINGS_FILE_NAME);
        let expected = UserSettings {
            last_input: "可恢复内容👍".to_string(),
            ..UserSettings::default()
        };

        fs::create_dir_all(&dir).unwrap();
        save_to(&path, &expected).expect("first save should succeed");
        save_to(&path, &UserSettings::default()).expect("second save should succeed");
        fs::write(&path, "invalid: [").unwrap();

        let loaded = load_from_dir_with_status(&dir).expect("backup should be loaded");
        assert_eq!(loaded.settings, expected);
        assert_eq!(
            loaded.notices,
            vec![SettingsLoadNotice::PrimarySettingsCorruptRecoveredFromBackup]
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn save_to_missing_directory_returns_diagnostic_error() {
        let path = std::env::temp_dir()
            .join(format!("ccw-missing-dir-{}", std::process::id()))
            .join(SETTINGS_FILE_NAME);
        let settings = UserSettings::default();
        let error = save_to(&path, &settings).expect_err("save should fail for missing dir");
        assert!(error.contains("settings directory does not exist"));
        assert!(error.contains(SETTINGS_FILE_NAME));
    }

    /// 旧版 ccw-formatter-settings.json（JSON）应被自动迁移为 rules.yaml。
    #[test]
    fn legacy_json_settings_are_migrated() {
        let dir = std::env::temp_dir().join(format!(
            "ccw-settings-migrate-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let expected = UserSettings {
            enabled: vec!["中英文之间需要增加空格".to_string()],
            last_input: "在LeanCloud上".to_string(),
            theme: ThemeMode::Light,
            font: FontFamily::System,
            editor_font_size: EditorFontSize::Normal,
            ui_scale: UiScale::Normal,
            shortcuts: ShortcutSettings::default(),
        };
        fs::write(
            dir.join(LEGACY_SETTINGS_FILE_NAME),
            serde_json::to_string(&expected).unwrap(),
        )
        .unwrap();
        assert_eq!(load_from_dir(&dir), Some(expected.clone()));
        // 迁移后 rules.yaml 存在且内容一致；旧 JSON 不再被读取。
        let migrated = load_from(&dir.join(SETTINGS_FILE_NAME)).expect("rules.yaml should exist");
        assert_eq!(migrated, expected);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn legacy_settings_report_migration_notice() {
        let dir = std::env::temp_dir().join(format!(
            "ccw-settings-notice-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join(LEGACY_SETTINGS_FILE_NAME),
            serde_json::to_string(&UserSettings::default()).unwrap(),
        )
        .unwrap();
        let loaded = load_from_dir_with_status(&dir).expect("legacy settings should load");
        assert_eq!(
            loaded.notices,
            vec![SettingsLoadNotice::LegacySettingsDetected]
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_primary_and_backup_report_both_notices() {
        let dir =
            std::env::temp_dir().join(format!("ccw-settings-corrupt-both-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join(SETTINGS_FILE_NAME), "invalid: [").unwrap();
        fs::write(dir.join(SETTINGS_BACKUP_FILE_NAME), "invalid: [").unwrap();
        let loaded = load_from_dir_with_status(&dir).expect("corrupt files should report status");
        assert_eq!(
            loaded.notices,
            vec![
                SettingsLoadNotice::PrimarySettingsCorruptNoUsableBackup,
                SettingsLoadNotice::BackupSettingsCorrupt
            ]
        );
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn old_yaml_defaults_new_display_settings() {
        let path = temp_settings_file("display-defaults");
        fs::write(&path, "enabled: []\nlast_input: ''\n").unwrap();
        let loaded = load_from(&path).expect("old yaml should parse");
        assert_eq!(loaded.editor_font_size, EditorFontSize::Normal);
        assert_eq!(loaded.ui_scale, UiScale::Normal);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn corrupt_file_returns_none() {
        let path = temp_settings_file("corrupt");
        fs::write(&path, "not json {{{\"").unwrap();
        assert_eq!(load_from(&path), None);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn partial_json_fills_defaults() {
        let path = temp_settings_file("partial");
        fs::write(&path, r#"{"enabled": ["a"]}"#).unwrap();
        let loaded = load_from(&path).expect("should parse");
        assert_eq!(loaded.enabled, vec!["a".to_string()]);
        assert_eq!(loaded.last_input, "");
        assert_eq!(loaded.theme, ThemeMode::System);
        assert_eq!(loaded.font, FontFamily::System);
        let _ = fs::remove_file(&path);
    }

    /// UTF-8 回归：设置文件必须以 UTF-8 写入/读回，emoji 与 CJK 不损坏。
    #[test]
    fn roundtrip_preserves_utf8_multibyte() {
        let path = temp_settings_file("utf8");
        let settings = UserSettings {
            enabled: vec![
                "中英文之间需要增加空格".to_string(),
                "简体中文使用直角引号".to_string(),
            ],
            last_input: "在LeanCloud上，花了5000元👍𠀀".to_string(),
            theme: ThemeMode::Dark,
            font: FontFamily::NotoSansCjk,
            editor_font_size: EditorFontSize::Normal,
            ui_scale: UiScale::Normal,
            shortcuts: ShortcutSettings {
                enabled: true,
                bindings: default_shortcut_bindings(),
            },
        };
        save_to(&path, &settings).expect("save should succeed");
        // 文件字节必须是合法 UTF-8 且包含原始字符。
        let raw = fs::read_to_string(&path).expect("settings file must be valid UTF-8");
        assert!(raw.contains("在LeanCloud上，花了5000元👍𠀀"));
        assert_eq!(load_from(&path), Some(settings));
        let _ = fs::remove_file(&path);
    }

    /// 无 theme 字段的旧版设置应默认回退为 System。
    #[test]
    fn theme_defaults_to_system() {
        let path = temp_settings_file("theme-default");
        fs::write(&path, "enabled: ['rule-a']\nlast_input: 'test'\n").unwrap();
        let loaded = load_from(&path).expect("should parse");
        assert_eq!(loaded.theme, ThemeMode::System);
        let _ = fs::remove_file(&path);
    }

    /// 缺少 shortcuts 字段的旧 YAML 应默认启用并使用内置绑定。
    #[test]
    fn old_yaml_defaults_shortcuts_enabled() {
        let path = temp_settings_file("shortcuts-default");
        fs::write(&path, "enabled: []\nlast_input: ''\n").unwrap();
        let loaded = load_from(&path).expect("old yaml should parse");
        assert!(loaded.shortcuts.enabled);
        assert_eq!(loaded.shortcuts.bindings, default_shortcut_bindings());
        let _ = fs::remove_file(&path);
    }

    /// 总开关与自定义绑定必须完整 round-trip。
    #[test]
    fn shortcut_settings_roundtrip() {
        let path = temp_settings_file("shortcuts-roundtrip");
        let mut settings = UserSettings::default();
        settings.shortcuts.enabled = false;
        settings
            .shortcuts
            .bindings
            .insert("format_now".to_string(), "CtrlOrCmd+Shift+KeyF".to_string());
        save_to(&path, &settings).expect("save should succeed");
        let loaded = load_from(&path).expect("should parse");
        assert_eq!(loaded.shortcuts, settings.shortcuts);
        let raw = fs::read_to_string(&path).expect("settings file must be readable");
        assert!(raw.contains("shortcuts"));
        assert!(raw.contains("CtrlOrCmd+Shift+KeyF"));
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(backup_path_for(&path));
    }

    /// 仅有 enabled、缺失 bindings 的 shortcuts 字段应补齐默认绑定。
    #[test]
    fn partial_shortcuts_fill_default_bindings() {
        let path = temp_settings_file("shortcuts-partial");
        fs::write(
            &path,
            "enabled: []\nlast_input: ''\nshortcuts:\n  enabled: false\n",
        )
        .unwrap();
        let loaded = load_from(&path).expect("should parse");
        assert!(!loaded.shortcuts.enabled);
        assert_eq!(loaded.shortcuts.bindings, default_shortcut_bindings());
        let _ = fs::remove_file(&path);
    }
}
