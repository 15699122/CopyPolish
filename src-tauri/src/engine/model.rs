// engine/model.rs —— 引擎核心数据模型。

/// 规则元数据（serde 序列化给 Tauri 前端）。
/// `key` 是稳定的机器标识（如 `spacing.cjk-latin`），展示名与分组仅用于 UI。
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct RuleMeta {
    pub key: String,
    pub section: String,
    pub name: String,
    pub disputed: bool,
    pub default: bool,
}

/// 规则选择模式，避免用空数组同时表达“全部启用”和“全部关闭”。
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum RuleSelection {
    /// 执行全部已注册规则。
    All,
    /// 执行注册表中标记为默认启用的规则。
    Defaults,
    /// 仅执行指定规则；未知 key 会被安全忽略。
    Only { keys: Vec<String> },
    /// 不执行任何规则。
    None,
}

/// 格式化请求。规则选择必须通过显式的 `selection` 表达。
pub struct FormatRequest {
    pub text: String,
    pub selection: RuleSelection,
}
