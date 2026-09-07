# glib 0.18.5 soundness 风险接受（GHSA-wrw7-89jp-8q8g）

> 记录日期：2026-09-07。本文件是 `verify.py --profile audit` 之外、针对 GitHub Dependabot
> 告警 #1 的正式风险接受登记，与 [e2e-audit-policy.json](e2e-audit-policy.json) 的
> E2E 登记相互独立。

## 告警信息

| 项 | 内容 |
| --- | --- |
| Advisory | GHSA-wrw7-89jp-8q8g（无 CVE） |
| 严重性 | Medium（Dependabot）；RustSec 归类为 unsound warning |
| 依赖 | `glib 0.18.5`（`src-tauri/Cargo.lock`） |
| 依赖链 | `tauri → tao/tauri-runtime-wry/wry/muda → gtk → gdk/cairo/gio/atk/pango/… → glib 0.18.5` |
| 修复版本 | `glib 0.20.0`（GTK 0.18 系列无法升级到该版本） |
| 范围 | 仅 Linux GUI 生产资产（`.deb`/`.rpm`/AppImage）；Windows 资产不链接 GTK |
| owner | CopyPolish maintainers |
| 复核期限 | 2026-12-31（或 Tauri/Wry GTK 栈提供 0.20 兼容升级时提前处置） |

## 问题描述

`glib::VariantStrIter` 的 `Iterator`/`DoubleEndedIterator` 实现存在 soundness
缺陷：向 C 变参函数传递 `&p` 而非 `&mut p` 作为出参，在新版 Rust 编译器优化下
会导致 `CStr::from_ptr` 的安全前提被破坏，可能触发 NULL 指针解引用崩溃。

## 风险评估

1. **不可达性**：CopyPolish 代码不直接使用 `glib::VariantStrIter`；该类型仅在
   GTK/GIO 内部解析 `GVariant` 字符串数组时使用。应用未通过 glib API 构造
   `VariantStrIter`。
2. **影响形态**：即使触发，表现为进程崩溃（可用性），不是内存写入或信息泄露。
3. **无缓解升级路径**：修复版本 0.20.0 属于 gtk-rs 0.20 代系，需要 Tauri/Wry
   完成 GTK 0.20 迁移；在 v0.6.2 维护版本范围内跨代升级超出安全维护边界。
4. **验证覆盖**：Linux GUI smoke（启动、格式化、设置保存、退出）作为发布
   演练的必经步骤，可捕获由该缺陷引起的启动期崩溃。

## 决定

- **限期接受**该告警，不视为发布阻断项；roadmap 的"生产依赖无未解释的
  high/critical 漏洞"门槛不受影响（本告警为 medium/unsound）。
- 保持 `cargo audit` 的 warning 计数透明登记（不使用 ignore 列表隐藏）。
- 当 Tauri 依赖栈提供 `glib 0.20+` 兼容路径，或 2026-12-31 复核到期时，
  必须重新评估并更新本文件。
