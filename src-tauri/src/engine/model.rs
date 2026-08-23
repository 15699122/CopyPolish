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

/// 格式化请求。约定：`enabled` 为空数组表示全部启用；
/// 含未知 key 时安全忽略（不报错），便于旧设置平滑迁移。
pub struct FormatRequest {
    pub text: String,
    pub enabled: Vec<String>,
}
