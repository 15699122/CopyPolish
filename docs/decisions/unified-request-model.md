# 统一请求模型：自定义替换、预设与简繁转换

## 状态

已采纳。

## 背景

CopyPolish 从单一中文排版工具扩展为「文本清洗与规范排版」工具后，需要承载三类运行时参数化操作：

1. **自定义字面量替换**（`cleanup.reference-square` 之外的批量查找替换）；
2. **简繁转换**（互斥的 `none` / `t2s` / `s2t` 模式）；
3. **预设**（中文文案 / PDF 清洗 / 技术文档等快捷配置）。

当前 `FormatRequest` 只包含 `text` 与 `selection`，无法承载这些参数。如果把它们伪装成静态 `RuleDef`（注册表条目），会破坏 `resolve_execution_order` 的依赖图，也让核心 phase 顺序面临被任意拖拽覆盖的风险。

## 决策

扩展 `FormatRequest` 为统一请求模型：

```rust
pub struct FormatRequest {
    pub text: String,
    pub selection: RuleSelection,
    pub replacements: Vec<ReplacementPair>,
    pub conversion: CharacterConversion,
}

pub struct ReplacementPair {
    pub from: String,
    pub to: String,
    pub active: bool,
}

pub enum CharacterConversion {
    None,
    TraditionalToSimplified,
    SimplifiedToTraditional,
}

pub struct Preset {
    pub key: String,
    pub name: String,
    pub description: String,
    pub selection: RuleSelection,
    pub replacements: Vec<ReplacementPair>,
    pub conversion: CharacterConversion,
}
```

### 关键约束

1. **预设只展开为统一请求模型，不复制规则实现。** 预设是 `{ selection, replacements, conversion }` 的命名模板，调用方通过 `preset.to_request(text)` 得到 `FormatRequest`，再走同一个 `format_text` 入口。
2. **核心 phase 与依赖图保持不变。** 自定义替换与简繁转换是独立的「请求层阶段」，不进入 `RulePhase` 枚举，也不参与 `resolve_execution_order` 拓扑排序。用户不能通过拖拽改变「保护 → 排版」这一核心顺序。
3. **替换在 span 保护前执行。** 避免自定义替换破坏 Markdown / URL / 代码结构；替换结果再进入 span 扫描与保护层。
4. **简繁转换互斥。** 由 `CharacterConversion` 枚举在请求层保证，不依赖运行时校验。
5. **默认行为不变。** 旧调用方只传 `{ text, selection }`，新字段默认为空 / `None`，输出与扩展前一致。

## 管线执行顺序（更新 architecture.md §3）

1. 归一化换行符
2. **自定义字面量替换**（有序、受 span 保护约束）
3. 跨行来源清洗（连续空行）
4. 可编辑区间清洗 / 标点 / 名词规则
5. **字符转换**（简繁，互斥）
6. 扫描 span → 保护占位符
7. 结构边界 / 文本边界 / 排版规则
8. 占位符边缘空格 → 还原

## 接入点

- `model.rs`：新增 `ReplacementPair`、`CharacterConversion`、`Preset`，扩展 `FormatRequest`
- `pipeline.rs`：在 `format_text_impl` 的归一化后、span 保护前插入替换与转换阶段
- `commands.rs`：`format_text` command 透传新字段
- `frontend/src/lib/tauri.ts`：`FormatRequest` 类型对齐
- 测试：新增 `replacement-and-conversion.yaml` fixture，覆盖替换顺序、简繁互斥、与排版规则组合

## 验证

- 回归：默认 `{ text, selection }` 调用输出与扩展前一致
- 新行为：自定义替换在 span 保护前生效、简繁互斥、预设可展开为请求模型
- 文档：`architecture.md` §3 / §4、`README.md`、`CHANGELOG.md`、`roadmap.md` 同步更新

## 备注

- 首版自定义替换**不支持用户正则**（roadmap §P1 明确）
- 简繁转换的具体 Rust 依赖与词汇级语义留待独立 Spike；模型层先占位 `CharacterConversion::None` 为唯一实际生效值
