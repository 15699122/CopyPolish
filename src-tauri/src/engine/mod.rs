// engine/mod.rs
// =============================================================================
// 可扩展文字处理引擎（v2 架构）。
//
// 设计目标：彻底解除对历史 Python 参考实现、固定规则
// 数量以及 Rust/Python parity 的架构依赖。历史 12 条规则的全部用户可见效果
// 被迁移为独立注册的规则模块；新增规则只需实现处理函数并加入 registry，
// 无需改动 command 层、格式化主循环或前端。
//
// 模块职责：
//   model      规则元数据 / 请求结构
//   tokenizer  Unicode 字符分类、特殊字符（上下标、化学式）识别
//   protection Markdown / LaTeX / URL / 邮箱 / 化学式保护层
//   rules      各条规则的纯函数实现
//   registry   规则注册表：稳定机器 key、元数据、默认启用、旧 key 迁移
//   pipeline   格式化主流程：保护 -> 逐行应用规则 -> 还原
// =============================================================================

pub mod model;
pub mod pipeline;
pub mod protection;
pub mod registry;
mod rule_impls;
// TextEdit 迁移脚手架；当前由模块内单元测试验证，尚未接管生产 pipeline。
#[allow(dead_code)]
pub(crate) mod edit_plan;
pub(crate) mod semantic_tokens;
// Span/edit 重构的迁移脚手架；当前由模块内单元测试验证，尚未接管生产 pipeline。
#[allow(dead_code)]
pub(crate) mod spans;
pub mod tokenizer;
pub(crate) mod unicode_boundaries;
pub(crate) mod unit_lexicon;

pub use model::{FormatRequest, RuleMeta, RuleSelection};
pub use pipeline::format_text;
pub use registry::{
    default_rules, enabled_defaults, execution_rules, normalize_rule_keys, rules, RuleDef,
    RulePhase,
};
pub use tokenizer::detect_chemical_formulas;

#[cfg(test)]
mod tests;
