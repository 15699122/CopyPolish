// engine/model.rs —— 引擎核心数据模型。

/// 规则类型：区分来源文本清洗、字符转换和规范排版。
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuleKind {
    Cleanup,
    Conversion,
    Typography,
}

impl RuleKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cleanup => "cleanup",
            Self::Conversion => "conversion",
            Self::Typography => "typography",
        }
    }
}

/// 规则风险等级：用于 UI 提示和后续预设准入，不改变当前执行语义。
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuleRisk {
    Safe,
    Contextual,
    Destructive,
}

impl RuleRisk {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Contextual => "contextual",
            Self::Destructive => "destructive",
        }
    }
}

/// 规则的单条修改示例：`before` 经该规则（仅启用此规则）处理后的结果是 `after`。
/// `after` 与注册表实现一致，不会随时间漂移；由 Rust 测试强制校验。
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct RuleExample {
    pub before: String,
    pub after: String,
}

/// 规则元数据（serde 序列化给 Tauri 前端）。
/// `key` 是稳定的机器标识（如 `spacing.cjk-latin`），展示名与分组仅用于 UI。
#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct RuleMeta {
    pub key: String,
    pub section: String,
    pub name: String,
    pub description: String,
    pub example: RuleExample,
    pub kind: RuleKind,
    pub risk: RuleRisk,
    pub disputed: bool,
    pub default: bool,
}

/// 规则选择模式，避免用空数组同时表达“全部启用”和“全部关闭”。
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq, Default)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum RuleSelection {
    /// 执行全部已注册规则。
    All,
    /// 执行注册表中标记为默认启用的规则。
    #[default]
    Defaults,
    /// 仅执行指定规则；未知 key 会被安全忽略。
    Only { keys: Vec<String> },
    /// 不执行任何规则。
    None,
}

/// 自定义字面量替换对（有序）。
///
/// 首版不支持用户正则；`from` 为字面量字符串，按向量顺序依次应用。
/// 替换在 span 保护前执行，避免破坏 Markdown / URL / 代码结构。
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ReplacementPair {
    pub from: String,
    pub to: String,
    pub active: bool,
}

/// 字符转换模式。简转繁与繁转简互斥，由请求模型在类型层面保证。
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum CharacterConversion {
    #[default]
    None,
    #[serde(rename = "t2s")]
    TraditionalToSimplified,
    #[serde(rename = "s2t")]
    SimplifiedToTraditional,
}

impl CharacterConversion {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::TraditionalToSimplified => "t2s",
            Self::SimplifiedToTraditional => "s2t",
        }
    }
}

impl Default for ReplacementPair {
    fn default() -> Self {
        Self {
            from: String::new(),
            to: String::new(),
            active: true,
        }
    }
}

/// 格式化请求。规则选择必须通过显式的 `selection` 表达。
///
/// `replacements` 与 `conversion` 为可选的请求层阶段：默认空 / None 时
/// 输出与扩展前完全一致，旧调用方可只传 `{ text, selection }` 并配合
/// `.. Default::default()` 填充新字段。
pub struct FormatRequest {
    pub text: String,
    pub selection: RuleSelection,
    pub replacements: Vec<ReplacementPair>,
    pub conversion: CharacterConversion,
}

impl Default for FormatRequest {
    fn default() -> Self {
        Self {
            text: String::new(),
            selection: RuleSelection::Defaults,
            replacements: Vec::new(),
            conversion: CharacterConversion::None,
        }
    }
}

/// 预设：统一请求模型的命名模板。
///
/// 预设只展开为 `FormatRequest`，不复制规则实现。核心 phase 与依赖图
/// （`resolve_execution_order`）保持不变，用户不能通过预设拖拽改变
/// 「保护 → 排版」这一核心顺序。
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct Preset {
    pub key: String,
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub selection: RuleSelection,
    #[serde(default)]
    pub replacements: Vec<ReplacementPair>,
    #[serde(default)]
    pub conversion: CharacterConversion,
}

impl Preset {
    /// 将预设展开为指定输入文本的格式化请求。
    pub fn to_request(&self, text: String) -> FormatRequest {
        FormatRequest {
            text,
            selection: self.selection.clone(),
            replacements: self.replacements.clone(),
            conversion: self.conversion,
        }
    }
}
