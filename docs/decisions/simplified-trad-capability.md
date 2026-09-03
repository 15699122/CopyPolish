# 简繁转换构建能力与 GUI 语义决策

## 状态

已采纳（2026-09-03）。本决策定义默认构建与 `simplified-trad-conversion` feature 构建在 GUI 中的能力声明和用户可见行为；实际 feature 依赖与转换算法仍以 [simplified-trad-conversion-spike.md](simplified-trad-conversion-spike.md) 为准。

## 1. 背景

CopyPolish 的 `FormatRequest` 已包含互斥的 `none` / `t2s` / `s2t`，并已通过可选 Cargo feature `simplified-trad-conversion` 接入 `opencc-fmmseg`。默认构建为了控制二进制体积和 native `zstd-sys` 构建成本不启用该依赖；此前 GUI 仍能显示并保存 t2s/s2t 选择，造成默认构建语义歧义，本次通过 capability 接线修复。

如果默认构建继续允许用户选择一个实际不会改变输出的模式，用户会误以为转换已经生效；这与“设置状态应反映真实行为”的要求不一致。另一方面，直接把 feature 加入所有默认发布构建会增加二进制体积、构建时间、许可证清单和跨平台发布验证范围，不能在没有独立发布验收的情况下隐式改变。

## 2. 决策

采用“**构建 capability 显式声明，默认构建不静默承诺转换**”方案：

1. Rust/Tauri 增加只读 capability 查询，至少返回：

   ```json
   {
     "simplifiedTradConversion": true
   }
   ```

2. `simplifiedTradConversion` 由 Cargo feature 编译条件决定：
   - 启用 `simplified-trad-conversion`：返回 `true`；
   - 默认构建：返回 `false`。

3. GUI 加载 capability 后：
   - `true`：启用 `none` / `t2s` / `s2t` 选择，正常保存和格式化；
   - `false`：保留当前设置说明，但禁用 t2s/s2t，显示“当前构建未包含简繁转换能力”；保存时将模式保持为 `none`，不允许产生一个静默无效的 t2s/s2t 设置。

4. 已存在的默认构建 `rules.yaml` 若包含 t2s/s2t：读取时保留兼容性，但 GUI 应展示能力不可用并在用户修改设置时归一化为 `none`；不能在加载阶段无提示地破坏用户文件。

5. 预览浏览器不宣称简繁能力。浏览器 fallback 只提供 UI 开发所需的最小行为，并显示演示模式和能力不可用提示。

6. 发布策略暂不改变：默认便携构建仍不启用 feature；是否把 feature 纳入正式 Windows/Linux 发布资产，另以发布验收结果决定。

## 3. 数据流和接入点

```text
Cargo feature
  → get_build_capabilities command
  → frontend/src/lib/tauri.ts
  → useSettingsLoader / useAppController
  → ReplacementsSection 能力提示与选择器禁用状态
  → FormatRequest conversion
```

已实现接入文件：

- `src-tauri/src/commands.rs` 与 Tauri command 注册入口：实现并注册 `get_build_capabilities`；
- `frontend/src/lib/tauri.ts`：实现 `BuildCapabilities` 查询封装；
- `frontend/src/hooks/useSettingsLoader.ts`：加载 capability，并在 E2E 诊断中记录；
- `frontend/src/hooks/useAppController.ts`：统一计算有效 conversion 并传递给设置动作/弹窗；
- `frontend/src/components/settings/ReplacementsSection.tsx` 与 `SettingsDialog.tsx`：禁用不可用方向并显示说明；
- `frontend/src/hooks/useSettingsActions.ts`：阻止默认构建保存无效 conversion；
- Rust/前端/E2E 测试：覆盖默认和 feature 两种构建。

## 4. 验收标准

### 默认构建

- capability 返回 `simplifiedTradConversion: false`；
- 设置窗口明确显示能力不可用；
- t2s/s2t 不能被选择或保存为有效当前模式；
- `conversion: none` 的默认格式化行为不变；
- 旧设置不会导致崩溃或静默改写；
- 浏览器演示模式不伪造 feature 能力。

### Feature 构建

- capability 返回 `simplifiedTradConversion: true`；
- s2t/t2s 可选择、保存和恢复；
- 真实 Rust IPC 输出保持：
  - `设计软件与打印` → `設計軟件與打印`；
  - `後設資料與說明` → `后设资料与说明`；
- 结构保护、设置重启和 E2E 保存序号诊断继续通过。

### 跨平台

- Linux/WSL 默认和 feature GUI E2E 均通过；
- Windows 默认和 feature GUI E2E 均通过；
- 默认、TUI、feature、组合 Cargo 测试通过；
- `docs/licenses.md` 与发布资产说明在正式启用 feature 时同步更新。

## 5. 不在本决策范围内

- 不决定是否最终让所有正式发布资产默认启用 feature；
- 不改变 OpenCC 词典的地区词语义；
- 不实现全角/半角或 Unicode 等价字符转换；
- 不把简繁转换接入 TUI/CLI；TUI/CLI 仍按 roadmap 单独设计。

## 6. 后续重新评估条件

- 正式发布用户普遍需要开箱即用的简繁转换；
- feature 构建在所有目标平台均稳定，体积/构建时间成本可接受；
- 发布流水线可以为默认和 feature 资产提供清晰命名及许可证清单；
- capability 方案导致 GUI 配置复杂度高于直接默认启用 feature。