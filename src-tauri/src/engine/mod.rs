// engine/mod.rs
// =============================================================================
// 可扩展文字处理引擎。
//
// 规则行为由 Rust 注册表和纯函数实现驱动；新增规则只需实现处理函数并加入
// registry，无需改动 command 层、格式化主循环或前端。
//
// 模块职责：
//   model      规则元数据 / 请求结构
//   tokenizer  Unicode 字符分类、特殊字符（上下标、化学式）识别
//   protection Markdown / LaTeX / URL / 邮箱 / 化学式保护层
//   rules      各条规则的纯函数实现
//   registry   规则注册表：稳定机器 key、元数据、默认启用、旧 key 迁移
//   pipeline   格式化主流程：保护 -> TextEdit 应用规则 -> 还原
// =============================================================================

pub mod model;
pub mod pipeline;
pub mod protection;
pub mod registry;
mod rule_impls;
// TextEdit 迁移模块：全部规则阶段（标点/名词/结构边界/文本边界/清理）均已
// 通过本模块的 TextEdit 应用层执行；占位符仍由 pipeline 的保护层承载。
#[allow(dead_code)]
pub(crate) mod edit_plan;
pub(crate) mod semantic_tokens;
// Span/edit 基础模块；span-aware 管线已接管生产入口。
#[allow(dead_code)]
pub(crate) mod spans;
pub mod tokenizer;
pub(crate) mod unicode_boundaries;
pub(crate) mod unit_lexicon;

pub use model::{FormatRequest, RuleMeta, RuleSelection};
pub use pipeline::format_text;
#[cfg(feature = "profile-stages")]
pub use pipeline::{format_text_stage_timings, per_rule_timings, scan_split_timings};
pub use registry::{
    default_rules, enabled_defaults, execution_rules, normalize_rule_keys, rules, RuleDef,
    RulePhase,
};
pub use tokenizer::detect_chemical_formulas;

#[cfg(test)]
mod tests;
