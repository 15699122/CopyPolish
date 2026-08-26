# 历史参考资料

本目录保存项目早期 Python 实现及其规则目录，仅用于理解历史迁移背景和必要的行为对照。

## 当前状态

- `ccw_engine.py` 和 `rule_catalog.yaml` 不参与当前 Tauri/Rust 构建、打包、测试或 CI 门禁；
- 不应以本目录内容作为新增规则或修复行为的唯一依据；
- 当前实现的事实来源是 `src-tauri/src/engine/`、`src-tauri/src/engine/registry.rs` 以及 `src-tauri/tests/fixtures/`；
- 前端访问后端的唯一封装入口是 `frontend/src/lib/tauri.ts`。

项目当前采用纯 Rust 排版引擎。旧版 Python 设置文件仅作为一次性迁移输入，当前运行时设置文件是程序同目录的 `rules.yaml`。
