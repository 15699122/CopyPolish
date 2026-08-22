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
/// 旧版设置文件名（JSON，仅用于一次性迁移读取）。
pub const LEGACY_SETTINGS_FILE_NAME: &str = "ccw-formatter-settings.json";

/// 主题模式：跟随系统 / 浅色 / 深色。
/// `#[serde(default)]` 确保旧版设置文件（无 theme 字段）可反序列化，
/// 默认回退为 `System`。
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::System
    }
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UserSettings {
    #[serde(default)]
    pub enabled: Vec<String>,
    #[serde(default)]
    pub last_input: String,
    #[serde(default)]
    pub theme: ThemeMode,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            enabled: Vec::new(),
            last_input: String::new(),
            theme: ThemeMode::default(),
        }
    }
}

/// 设置保存目录：优先使用当前可执行文件所在目录，失败时回退当前工作目录。
fn settings_dir() -> PathBuf {
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

    fs::rename(&tmp_path, path).map_err(|e| {
        let _ = fs::remove_file(&tmp_path);
        format!(
            "replace settings file {} with temporary file {} failed; directory is {}: {e}",
            path.display(),
            tmp_path.display(),
            dir.display()
        )
    })
}

/// 读取指定目录下的用户设置（含旧版 JSON 迁移逻辑），供测试注入目录。
pub fn load_from_dir(dir: &Path) -> Option<UserSettings> {
    let path = dir.join(SETTINGS_FILE_NAME);
    if let Some(settings) = load_from(&path) {
        return Some(settings);
    }
    let legacy = dir.join(LEGACY_SETTINGS_FILE_NAME);
    if legacy.exists() {
        if let Ok(json) = fs::read_to_string(&legacy) {
            if let Ok(settings) = serde_json::from_str::<UserSettings>(&json) {
                // 迁移成功与否不影响本次返回；失败则下次再试。
                let _ = save_to(&path, &settings);
                return Some(settings);
            }
        }
    }
    None
}

/// 读取用户设置：
/// 1. 优先读 exe 同目录的 rules.yaml；
/// 2. 若不存在，尝试读取同目录旧版 ccw-formatter-settings.json（JSON）并
///    自动迁移写入 rules.yaml；
/// 3. 均不存在时返回 None。
pub fn load() -> Option<UserSettings> {
    load_from_dir(&settings_dir())
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
        };
        save_to(&path, &settings).expect("save should succeed");
        assert_eq!(load_from(&path), Some(settings));
        // 保存格式必须是 YAML（含键名 enabled / last_input / theme）。
        let raw = fs::read_to_string(&path).expect("settings file must be readable");
        assert!(raw.contains("enabled"));
        assert!(raw.contains("last_input"));
        assert!(raw.contains("theme"));
        let _ = fs::remove_file(&path);
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
}
