// user_settings.rs
// =============================================================================
// 用户设置持久化（ADR：docs/decisions/settings-storage-policy.md 方案 B）。
// 默认保存在「可执行文件所在目录」下的 rules.yaml（YAML 格式）；
// exe 目录不可写时，启动时一次性决策回退到平台应用数据目录
// （Windows %APPDATA%\CopyPolish、Linux/macOS ~/.config/CopyPolish），
// 并通过 UsingAppDataFallback 提醒前端展示实际生效路径。
//
// 迁移：若 rules.yaml 不存在但同目录存在旧版 ccw-formatter-settings.json，
// 自动读取旧 JSON 并转换写入新的 rules.yaml。
//
// 注意：单元测试一律使用系统临时目录中的随机文件，绝不写仓库内文件，
// 避免测试之间 / 测试与真实设置之间的写覆盖。
// =============================================================================

use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

use crate::engine::{
    CharacterConversion, ReplacementPair, MAX_REPLACEMENTS, MAX_REPLACEMENT_FIELD_BYTES,
    MAX_RULE_SELECTION_KEYS,
};

/// 用户设置文件名（exe 同目录，YAML 格式）。
pub const SETTINGS_FILE_NAME: &str = "rules.yaml";
/// 设置备份文件名；主文件损坏时作为最后一次有效保存的恢复来源。
pub const SETTINGS_BACKUP_FILE_NAME: &str = "rules.yaml.bak";
/// 设置文件解析前的最大字节数，避免异常文件导致无界内存分配。
pub const MAX_SETTINGS_FILE_BYTES: u64 = 2 * 1024 * 1024;
pub const MAX_SHORTCUT_BINDING_BYTES: usize = 128;

static TEMPORARY_FILE_COUNTER: AtomicU64 = AtomicU64::new(0);
static SETTINGS_SAVE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

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

/// 为设置保存生成进程/时间唯一的临时路径，避免多个进程共享固定
/// `rules.yaml.tmp` 时互相覆盖或误操作未知文件。
fn temporary_path_for(settings_path: &Path) -> PathBuf {
    let file_name = settings_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("rules.yaml");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let counter = TEMPORARY_FILE_COUNTER.fetch_add(1, Ordering::Relaxed);
    settings_path.with_file_name(format!(
        ".{file_name}.tmp-{}-{timestamp}-{counter}",
        std::process::id()
    ))
}

/// 不跟随链接检查设置写入目标。设置路径、备份路径和临时路径都必须
/// 是普通文件或尚不存在；否则拒绝操作，避免把内容写到非预期目标。
fn reject_symlink(path: &Path, label: &str) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!(
            "{label} path must not be a symbolic link: {}",
            path.display()
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "inspect {label} path {} failed: {error}",
            path.display()
        )),
    }
}

#[cfg(unix)]
fn set_private_permissions(file: &fs::File, path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;
    file.set_permissions(fs::Permissions::from_mode(0o600))
        .map_err(|error| {
            format!(
                "set private permissions on {} failed: {error}",
                path.display()
            )
        })
}

#[cfg(not(unix))]
fn set_private_permissions(_file: &fs::File, _path: &Path) -> Result<(), String> {
    Ok(())
}

#[cfg(unix)]
fn sync_directory(dir: &Path) -> Result<(), String> {
    fs::File::open(dir)
        .and_then(|file| file.sync_all())
        .map_err(|error| format!("sync settings directory {} failed: {error}", dir.display()))
}

#[cfg(not(unix))]
fn sync_directory(_dir: &Path) -> Result<(), String> {
    Ok(())
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

/// 输出更新模式：输入变化后实时排版，或由用户显式触发排版。
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum OutputMode {
    #[default]
    Realtime,
    Manual,
}

/// 主界面输入/输出布局。
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LayoutMode {
    #[default]
    Auto,
    Horizontal,
    Vertical,
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
    pub output_mode: OutputMode,
    #[serde(default)]
    pub layout_mode: LayoutMode,
    #[serde(default)]
    pub shortcuts: ShortcutSettings,
    /// 有序自定义字面量替换（请求层阶段，span 保护前执行）。
    #[serde(default)]
    pub replacements: Vec<ReplacementPair>,
    /// 简繁转换模式（互斥，默认 `None`）。
    #[serde(default)]
    pub conversion: CharacterConversion,
    /// 启动时是否恢复上次输入正文（隐私开关，默认关闭）。
    ///
    /// 关闭时：加载侧不得把 `last_input` 恢复到输入框；保存侧必须把
    /// `last_input` 清空后落盘，用户正文不进入 `rules.yaml`。
    /// `#[serde(default)]` 使旧版设置文件（无该字段）自动采用隐私默认。
    #[serde(default)]
    pub restore_last_input: bool,
}

/// 按隐私开关归一化设置中的用户正文：未开启恢复时清空 `last_input`。
///
/// 所有持久化入口（GUI command、TUI persist）在写入前必须调用，
/// 保证正文不会在用户未显式开启“恢复上次输入”时进入设置文件。
pub fn enforce_input_privacy(settings: &mut UserSettings) {
    if !settings.restore_last_input {
        settings.last_input = String::new();
    }
}

/// 校验用户设置资源规模。保存入口和文件序列化入口都应调用，避免
/// 恶意/损坏调用方通过设置文件制造无界的内存或磁盘增长。
pub fn validate_user_settings(settings: &UserSettings) -> Result<(), String> {
    if settings.enabled.len() > MAX_RULE_SELECTION_KEYS {
        return Err(format!(
            "enabled rules exceed the {} key limit",
            MAX_RULE_SELECTION_KEYS
        ));
    }
    if settings.replacements.len() > MAX_REPLACEMENTS {
        return Err(format!(
            "replacements exceed the {} item limit",
            MAX_REPLACEMENTS
        ));
    }
    for pair in &settings.replacements {
        if pair.from.len() > MAX_REPLACEMENT_FIELD_BYTES
            || pair.to.len() > MAX_REPLACEMENT_FIELD_BYTES
        {
            return Err(format!(
                "replacement fields exceed the {} KiB limit",
                MAX_REPLACEMENT_FIELD_BYTES / 1024
            ));
        }
    }
    for binding in settings.shortcuts.bindings.values() {
        if binding.len() > MAX_SHORTCUT_BINDING_BYTES {
            return Err(format!(
                "shortcut bindings exceed the {} byte limit",
                MAX_SHORTCUT_BINDING_BYTES
            ));
        }
    }
    Ok(())
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
    /// exe 目录不可写，设置已回退保存到平台应用数据目录。
    UsingAppDataFallback,
}

/// 设置加载结果及加载阶段产生的提醒。
#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct LoadedUserSettings {
    pub settings: UserSettings,
    pub notices: Vec<SettingsLoadNotice>,
}

/// 设置保存目录决策（ADR：docs/decisions/settings-storage-policy.md，方案 B）：
/// 1. E2E 注入目录优先（仅 e2e feature）；
/// 2. exe 同目录已存在任何设置文件（rules.yaml / .bak / 旧 JSON）→ 继续使用；
/// 3. exe 同目录可写 → 继续使用（现有便携用户行为不变）；
/// 4. exe 目录不可写 → 回退平台应用数据目录（Windows `%APPDATA%\CopyPolish`、
///    Linux/macOS `$XDG_CONFIG_HOME/CopyPolish` 或 `~/.config/CopyPolish`），
///    并标记 fallback 以便 UI 提示；
/// 5. 均不可用 → 维持 exe 目录，让保存错误按原诊断信息呈现。
///
/// 决策在进程内只做一次（OnceLock 缓存），不在保存时改变位置。
struct StorageDecision {
    dir: PathBuf,
    uses_app_data_fallback: bool,
}

static STORAGE: std::sync::OnceLock<StorageDecision> = std::sync::OnceLock::new();

/// 平台应用数据目录（不保证存在）。
fn app_data_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("APPDATA").map(|base| PathBuf::from(base).join("CopyPolish"))
    }
    #[cfg(not(target_os = "windows"))]
    {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))?;
        Some(base.join("CopyPolish"))
    }
}

/// 目录中是否已存在任何形式的设置文件。
fn dir_has_any_settings(dir: &Path) -> bool {
    dir.join(SETTINGS_FILE_NAME).exists()
        || dir.join(SETTINGS_BACKUP_FILE_NAME).exists()
        || dir.join(LEGACY_SETTINGS_FILE_NAME).exists()
}

/// 通过创建临时探针文件判断目录可写性；探针随后立即删除。
pub(crate) fn dir_is_writable(dir: &Path) -> bool {
    // GitLab Linux Runner 可能以 root 运行；root 可以绕过 Unix 的目录
    // 写保护并成功创建文件。先检查权限位，保持“明确标记为只读的目录”
    // 的语义，再用探针覆盖 ACL、挂载和其它实际文件系统限制。
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let Ok(metadata) = fs::metadata(dir) else {
            return false;
        };
        if metadata.permissions().mode() & 0o222 == 0 {
            return false;
        }
    }

    let probe = dir.join(format!(".copypolish-write-probe-{}", std::process::id()));
    match fs::File::create(&probe) {
        Ok(_) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

/// 纯决策函数：返回 (生效目录, 是否回退到应用数据目录)。供单元测试注入。
fn resolve_storage_dir(exe_dir: Option<&Path>, app_dir: Option<&Path>) -> (PathBuf, bool) {
    if let Some(exe) = exe_dir {
        if dir_has_any_settings(exe) {
            return (exe.to_path_buf(), false);
        }
        if dir_is_writable(exe) {
            return (exe.to_path_buf(), false);
        }
    }
    if let Some(app) = app_dir {
        if fs::create_dir_all(app).is_ok() && dir_is_writable(app) {
            return (app.to_path_buf(), true);
        }
    }
    (
        exe_dir
            .map(Path::to_path_buf)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
        false,
    )
}

fn storage() -> &'static StorageDecision {
    STORAGE.get_or_init(|| {
        #[cfg(any(feature = "e2e-wdio", feature = "e2e-webdriver"))]
        if let Ok(dir) = std::env::var("COPYPOLISH_E2E_SETTINGS_DIR") {
            if !dir.is_empty() {
                return StorageDecision {
                    dir: PathBuf::from(dir),
                    uses_app_data_fallback: false,
                };
            }
        }

        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(Path::to_path_buf));
        let (dir, fallback) = resolve_storage_dir(exe_dir.as_deref(), app_data_dir().as_deref());
        StorageDecision {
            dir,
            uses_app_data_fallback: fallback,
        }
    })
}

/// 设置文件所在目录（进程内决策一次）。
pub fn settings_dir() -> PathBuf {
    storage().dir.clone()
}

pub fn settings_path() -> PathBuf {
    storage().dir.join(SETTINGS_FILE_NAME)
}

/// 当前是否因 exe 目录不可写而回退到平台应用数据目录。
pub fn uses_app_data_fallback() -> bool {
    storage().uses_app_data_fallback
}

/// 从指定路径读取 YAML 设置；文件缺失或解析失败时返回 None（调用方自行回落默认值）。
pub fn load_from(path: &Path) -> Option<UserSettings> {
    let metadata = fs::metadata(path).ok()?;
    if !metadata.is_file() || metadata.len() > MAX_SETTINGS_FILE_BYTES {
        return None;
    }
    let text = fs::read_to_string(path).ok()?;
    let settings = serde_yaml::from_str(&text).ok()?;
    validate_user_settings(&settings).ok()?;
    Some(settings)
}

pub fn save_to(path: &Path, settings: &UserSettings) -> Result<(), String> {
    let _save_guard = SETTINGS_SAVE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .map_err(|_| "settings save lock is poisoned".to_string())?;
    validate_user_settings(settings)?;
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

    reject_symlink(path, "settings")?;
    let backup_path = backup_path_for(path);
    reject_symlink(&backup_path, "settings backup")?;
    let tmp_path = temporary_path_for(path);
    reject_symlink(&tmp_path, "temporary settings")?;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp_path)
        .map_err(|error| {
            format!(
                "create temporary settings file {} failed; target settings file is {}; directory is {}; error: {error}",
                tmp_path.display(),
                path.display(),
                dir.display()
            )
        })?;
    if let Err(error) = set_private_permissions(&file, &tmp_path) {
        drop(file);
        let _ = fs::remove_file(&tmp_path);
        return Err(error);
    }
    if let Err(error) = file.write_all(yaml.as_bytes()) {
        drop(file);
        let _ = fs::remove_file(&tmp_path);
        return Err(format!(
            "write temporary settings file {} failed; target settings file is {}: {error}",
            tmp_path.display(),
            path.display()
        ));
    }
    if let Err(error) = file.sync_all() {
        drop(file);
        let _ = fs::remove_file(&tmp_path);
        return Err(format!(
            "flush temporary settings file {} failed; target settings file is {}: {error}",
            tmp_path.display(),
            path.display()
        ));
    }
    drop(file);

    // 先保留当前有效文件，再替换主文件。若备份轮换失败，主文件仍保持不变。
    let mut backup_moved = false;
    if path.exists() {
        if backup_path.exists() {
            reject_symlink(&backup_path, "settings backup")?;
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
        if let Err(error) = set_private_permissions(
            &fs::File::open(&backup_path).map_err(|e| {
                format!("open settings backup {} failed: {e}", backup_path.display())
            })?,
            &backup_path,
        ) {
            let _ = fs::rename(&backup_path, path);
            let _ = fs::remove_file(&tmp_path);
            return Err(error);
        }
        backup_moved = true;
        sync_directory(dir)?;
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
    sync_directory(dir)?;
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
    let mut loaded = load_from_dir_with_status(&settings_dir())?;
    if uses_app_data_fallback() {
        loaded
            .notices
            .push(SettingsLoadNotice::UsingAppDataFallback);
    }
    Some(loaded)
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

    // ---- 存储目录决策（ADR 方案 B）----

    fn unique_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "ccw-storage-{}-{}-{}",
            tag,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn exe_dir_with_existing_settings_wins_over_app_dir() {
        // 双位置并存：exe 目录已有设置文件时必须继续使用 exe 目录。
        let exe = unique_dir("exe-priority");
        let app = unique_dir("app-priority");
        fs::write(exe.join(SETTINGS_FILE_NAME), "enabled: []\n").unwrap();
        fs::write(app.join(SETTINGS_FILE_NAME), "enabled: [stale]\n").unwrap();

        let (dir, fallback) = resolve_storage_dir(Some(&exe), Some(&app));
        assert_eq!(dir, exe);
        assert!(!fallback);
        let _ = fs::remove_dir_all(&exe);
        let _ = fs::remove_dir_all(&app);
    }

    #[cfg(unix)]
    #[test]
    fn readonly_exe_dir_falls_back_to_app_data_dir() {
        use std::os::unix::fs::PermissionsExt;
        let exe = unique_dir("exe-readonly");
        let app = unique_dir("app-readonly");
        fs::set_permissions(&exe, fs::Permissions::from_mode(0o555)).unwrap();

        let (dir, fallback) = resolve_storage_dir(Some(&exe), Some(&app));
        // 恢复权限以便清理。
        fs::set_permissions(&exe, fs::Permissions::from_mode(0o755)).unwrap();
        assert_eq!(dir, app);
        assert!(fallback);
        let _ = fs::remove_dir_all(&exe);
        let _ = fs::remove_dir_all(&app);
    }

    #[test]
    fn writable_exe_dir_is_preferred_without_settings() {
        let exe = unique_dir("exe-fresh");
        let app = unique_dir("app-fresh");
        let (dir, fallback) = resolve_storage_dir(Some(&exe), Some(&app));
        assert_eq!(dir, exe);
        assert!(!fallback);
        let _ = fs::remove_dir_all(&exe);
        let _ = fs::remove_dir_all(&app);
    }

    #[test]
    fn backup_or_legacy_files_also_pin_exe_dir() {
        let exe = unique_dir("exe-bak");
        let app = unique_dir("app-bak");
        fs::write(exe.join(SETTINGS_BACKUP_FILE_NAME), "enabled: []\n").unwrap();
        assert!(dir_has_any_settings(&exe));
        let (dir, fallback) = resolve_storage_dir(Some(&exe), Some(&app));
        assert_eq!(dir, exe);
        assert!(!fallback);

        let exe2 = unique_dir("exe-legacy");
        fs::write(exe2.join(LEGACY_SETTINGS_FILE_NAME), "{}").unwrap();
        let (dir2, _) = resolve_storage_dir(Some(&exe2), Some(&app));
        assert_eq!(dir2, exe2);
        let _ = fs::remove_dir_all(&exe);
        let _ = fs::remove_dir_all(&exe2);
        let _ = fs::remove_dir_all(&app);
    }

    #[test]
    fn unwritable_everywhere_keeps_exe_dir_and_reports_save_error() {
        let exe = unique_dir("exe-both-readonly");
        let app = unique_dir("app-both-readonly");

        // 不依赖 chmod 语义：把两个候选路径替换为普通文件，确保即使
        // 测试进程以 root 运行，也无法在其下创建设置文件。
        fs::remove_dir(&exe).unwrap();
        fs::remove_dir(&app).unwrap();
        fs::write(&exe, "exe path placeholder").unwrap();
        fs::write(&app, "app path placeholder").unwrap();

        let (dir, fallback) = resolve_storage_dir(Some(&exe), Some(&app));
        // 两个候选位置都不可用时，仍维持 exe 目录决策，并返回带路径的诊断错误。
        let error = save_to(&dir.join(SETTINGS_FILE_NAME), &UserSettings::default())
            .expect_err("save should fail when storage path is a file");
        assert!(error.contains("settings parent path is not a directory"));
        assert_eq!(dir, exe);
        assert!(!fallback);
        let _ = fs::remove_file(&exe);
        let _ = fs::remove_file(&app);
    }

    #[test]
    fn write_probe_detects_writability() {
        let dir = unique_dir("probe");
        assert!(dir_is_writable(&dir));
        // 探针文件不应残留。
        assert!(!dir_has_any_settings(&dir));
        let _ = fs::remove_dir_all(&dir);
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
            restore_last_input: true,
            theme: ThemeMode::Dark,
            font: FontFamily::Pingfang,
            editor_font_size: EditorFontSize::Large,
            ui_scale: UiScale::Small,
            output_mode: OutputMode::Manual,
            layout_mode: LayoutMode::Vertical,
            shortcuts: ShortcutSettings {
                enabled: false,
                bindings: default_shortcut_bindings(),
            },
            replacements: vec![ReplacementPair {
                from: "TODO".to_string(),
                to: "待办".to_string(),
                active: true,
            }],
            conversion: CharacterConversion::SimplifiedToTraditional,
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
        assert!(raw.contains("output_mode"));
        assert!(raw.contains("layout_mode"));
        assert!(raw.contains("replacements"));
        assert!(raw.contains("conversion"));
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
    fn temporary_paths_are_unique() {
        let path = temp_settings_file("temporary-path");
        let first = temporary_path_for(&path);
        let second = temporary_path_for(&path);
        assert_ne!(first, second);
    }

    #[test]
    fn concurrent_saves_leave_parseable_settings_and_backup() {
        use std::sync::Arc;
        use std::thread;

        let path = Arc::new(temp_settings_file("concurrent-save"));
        let workers: Vec<_> = (0..8)
            .map(|index| {
                let path = Arc::clone(&path);
                thread::spawn(move || {
                    let settings = UserSettings {
                        enabled: vec![format!("rule-{index}")],
                        restore_last_input: true,
                        last_input: format!("内容-{index}"),
                        ..UserSettings::default()
                    };
                    save_to(path.as_ref(), &settings)
                })
            })
            .collect();

        for worker in workers {
            worker
                .join()
                .expect("concurrent save worker must not panic")
                .expect("serialized concurrent save should succeed");
        }

        assert!(load_from(path.as_ref()).is_some());
        assert!(load_from(&backup_path_for(path.as_ref())).is_some());

        let parent = path.parent().unwrap();
        let leftovers: Vec<_> = fs::read_dir(parent)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name())
            .filter(|name| name.to_string_lossy().contains("concurrent-save"))
            .collect();
        assert_eq!(leftovers.len(), 2, "only settings and backup should remain");
        let _ = fs::remove_file(path.as_ref());
        let _ = fs::remove_file(backup_path_for(path.as_ref()));
    }

    #[cfg(unix)]
    #[test]
    fn settings_and_backup_files_use_private_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let path = temp_settings_file("private-permissions");
        let backup = backup_path_for(&path);
        save_to(&path, &UserSettings::default()).expect("first save should succeed");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );

        save_to(&path, &UserSettings::default()).expect("second save should succeed");
        assert_eq!(
            fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(&backup).unwrap().permissions().mode() & 0o777,
            0o600
        );

        let leftovers: Vec<_> = fs::read_dir(path.parent().unwrap())
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.file_name())
            .filter(|name| name.to_string_lossy().contains("private-permissions"))
            .collect();
        assert_eq!(leftovers.len(), 2, "only settings and backup should remain");
        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&backup);
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_settings_target_is_rejected_without_modifying_target() {
        use std::os::unix::fs::symlink;

        let path = temp_settings_file("symlink-target");
        let target = temp_settings_file("symlink-target-real");
        fs::write(&target, "sentinel").unwrap();
        symlink(&target, &path).unwrap();

        let error = save_to(&path, &UserSettings::default()).expect_err("symlink must be rejected");
        assert!(error.contains("must not be a symbolic link"));
        assert_eq!(fs::read_to_string(&target).unwrap(), "sentinel");

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&target);
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_backup_target_is_rejected_without_modifying_existing_settings() {
        use std::os::unix::fs::symlink;

        let path = temp_settings_file("symlink-backup");
        let backup = backup_path_for(&path);
        let target = temp_settings_file("symlink-backup-real");
        save_to(&path, &UserSettings::default()).expect("first save should succeed");
        fs::write(&target, "sentinel").unwrap();
        symlink(&target, &backup).unwrap();

        let error =
            save_to(&path, &UserSettings::default()).expect_err("backup symlink must be rejected");
        assert!(error.contains("settings backup path must not be a symbolic link"));
        assert!(path.is_file());
        assert_eq!(fs::read_to_string(&target).unwrap(), "sentinel");

        let _ = fs::remove_file(&path);
        let _ = fs::remove_file(&backup);
        let _ = fs::remove_file(&target);
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
            restore_last_input: true,
            theme: ThemeMode::Light,
            font: FontFamily::System,
            editor_font_size: EditorFontSize::Normal,
            ui_scale: UiScale::Normal,
            output_mode: OutputMode::Realtime,
            layout_mode: LayoutMode::Auto,
            shortcuts: ShortcutSettings::default(),
            replacements: Vec::new(),
            conversion: CharacterConversion::None,
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
    fn input_privacy_defaults_off_and_enforced() {
        // 旧版 YAML 无 restore_last_input 字段：serde default 必须为 false（隐私优先）。
        let path = temp_settings_file("input-privacy-default");
        fs::write(&path, "enabled: []\nlast_input: '敏感正文'\n").unwrap();
        let loaded = load_from(&path).expect("should parse");
        assert!(!loaded.restore_last_input);

        // 关闭恢复时：enforce 必须清空正文。
        let mut settings = loaded;
        assert_eq!(settings.last_input, "敏感正文");
        enforce_input_privacy(&mut settings);
        assert_eq!(settings.last_input, "");

        // 开启恢复时：正文原样保留。
        let mut opt_in = UserSettings {
            last_input: "保留正文".to_string(),
            restore_last_input: true,
            ..UserSettings::default()
        };
        enforce_input_privacy(&mut opt_in);
        assert_eq!(opt_in.last_input, "保留正文");

        // 落盘归一化：关闭恢复保存后，文件中的 last_input 必须为空。
        let path = temp_settings_file("input-privacy-save");
        let mut saved = UserSettings {
            last_input: "不应落盘".to_string(),
            restore_last_input: false,
            ..UserSettings::default()
        };
        enforce_input_privacy(&mut saved);
        save_to(&path, &saved).expect("save should succeed");
        let reloaded = load_from(&path).expect("reload should parse");
        assert_eq!(reloaded.last_input, "");
        assert!(!reloaded.restore_last_input);
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
    fn oversized_settings_are_rejected_before_use() {
        let path = temp_settings_file("oversized-file");
        fs::write(&path, "x".repeat((MAX_SETTINGS_FILE_BYTES + 1) as usize)).unwrap();
        assert_eq!(load_from(&path), None);
        let _ = fs::remove_file(&path);

        let path = temp_settings_file("oversized-replacements");
        let settings = UserSettings {
            replacements: vec![ReplacementPair::default(); MAX_REPLACEMENTS + 1],
            ..UserSettings::default()
        };
        fs::write(&path, serde_yaml::to_string(&settings).unwrap()).unwrap();
        assert_eq!(load_from(&path), None);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn oversized_shortcut_binding_is_rejected() {
        let settings = UserSettings {
            shortcuts: ShortcutSettings {
                bindings: std::collections::BTreeMap::from([(
                    "format_now".to_string(),
                    "x".repeat(MAX_SHORTCUT_BINDING_BYTES + 1),
                )]),
                ..ShortcutSettings::default()
            },
            ..UserSettings::default()
        };
        assert!(validate_user_settings(&settings).is_err());
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
            restore_last_input: true,
            theme: ThemeMode::Dark,
            font: FontFamily::NotoSansCjk,
            editor_font_size: EditorFontSize::Normal,
            ui_scale: UiScale::Normal,
            output_mode: OutputMode::Realtime,
            layout_mode: LayoutMode::Auto,
            shortcuts: ShortcutSettings {
                enabled: true,
                bindings: default_shortcut_bindings(),
            },
            replacements: Vec::new(),
            conversion: CharacterConversion::None,
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
