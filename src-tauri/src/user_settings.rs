// user_settings.rs
// =============================================================================
// 用户设置持久化：保存在「当前工作目录」下的 JSON 文件。
// 与旧版 customtkinter GUI 的 rules.yaml 设置完全无关（旧 GUI 已废弃）。
//
// 注意：单元测试一律使用系统临时目录中的随机文件，绝不写仓库内文件，
// 避免测试之间 / 测试与真实设置之间的写覆盖。
// =============================================================================

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// 当前工作目录下的设置文件名。
pub const SETTINGS_FILE_NAME: &str = "ccw-formatter-settings.json";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct UserSettings {
    #[serde(default)]
    pub enabled: Vec<String>,
    #[serde(default)]
    pub last_input: String,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            enabled: Vec::new(),
            last_input: String::new(),
        }
    }
}

fn cwd() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

pub fn settings_path() -> PathBuf {
    cwd().join(SETTINGS_FILE_NAME)
}

/// 从指定路径读取；文件缺失或解析失败时返回 None（调用方自行回落默认值）。
pub fn load_from(path: &Path) -> Option<UserSettings> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn save_to(path: &Path, settings: &UserSettings) -> Result<(), String> {
    let json =
        serde_json::to_string_pretty(settings).map_err(|e| format!("serialize settings: {e}"))?;
    fs::write(path, json).map_err(|e| format!("write settings {}: {e}", path.display()))
}

/// 读取当前工作目录的设置；文件不存在返回 None。
pub fn load() -> Option<UserSettings> {
    load_from(&settings_path())
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
        path.push(format!("ccw-user-settings-test-{pid}-{n}-{tag}.json"));
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
        };
        save_to(&path, &settings).expect("save should succeed");
        assert_eq!(load_from(&path), Some(settings));
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn corrupt_file_returns_none() {
        let path = temp_settings_file("corrupt");
        fs::write(&path, "not json {{{").unwrap();
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
        };
        save_to(&path, &settings).expect("save should succeed");
        // 文件字节必须是合法 UTF-8 且包含原始字符。
        let raw = fs::read_to_string(&path).expect("settings file must be valid UTF-8");
        assert!(raw.contains("在LeanCloud上，花了5000元👍𠀀"));
        assert_eq!(load_from(&path), Some(settings));
        let _ = fs::remove_file(&path);
    }
}
