# 安全策略

## 支持的版本

| 版本 | 支持状态 |
| --- | --- |
| 0.6.x（含正在开发的 0.6.2 维护线） | ✅ 接收安全修复 |
| 0.5.x | ⚠️ 仅接收高危（high/critical）修复 |
| 更早版本 | ❌ 不再支持 |

## 如何报告漏洞

**请勿在公开 issue 中粘贴真实正文、token、私钥或未脱敏日志。**

本仓库已启用 GitHub **Private Vulnerability Reporting（私密漏洞报告）**。请优先通过以下入口私密报告：

1. 打开 <https://github.com/15699122/CopyPolish/security/advisories/new>；
2. 按模板填写受影响版本、平台、最小复现步骤和影响范围。

报告内容请包含：

- 受影响版本和平台（Windows / Linux，GUI / TUI / CLI）；
- 最小复现步骤；
- 影响范围（是否涉及用户正文、设置文件或凭据）；
- 是否包含真实个人数据；
- 推荐的私密联系方式（若问题本身包含敏感信息）。

## 响应时限

- **确认**：收到报告后 5 个工作日内确认收到并给出初步评估；
- **评估**：10 个工作日内给出影响分析和修复计划；
- **修复**：按严重程度排序，critical/high 优先发布补丁版本；
- 修复完成后会更新 [CHANGELOG.md](CHANGELOG.md)、[docs/security.md](docs/security.md) 和相关决策文档。

## 范围说明

以下内容**不属于**安全漏洞，请通过普通 issue 讨论：

- 浏览器演示模式不提供桌面版 Rust 引擎的行为等价性（这是设计边界，见 [docs/privacy.md](docs/privacy.md)）；
- 用户显式开启 `restore_last_input` 后正文以明文保存在本地 `rules.yaml`（已在使用说明中声明）；
- 需要本地物理访问或已泄露系统凭据的攻击场景。

## 安全模型与隐私说明

- 安全模型、信任边界与供应链门禁：[docs/security.md](docs/security.md)
- 用户数据处理说明：[docs/privacy.md](docs/privacy.md)
- E2E 依赖已接受风险登记：[docs/decisions/e2e-audit-policy.json](docs/decisions/e2e-audit-policy.json)

## 密钥与凭据

发布相关凭据使用 SOPS/age 管理，不进入 Git 历史。若发现疑似泄露的凭据：

1. 立即通过私密漏洞报告通知维护者；
2. 不要在公开渠道粘贴凭据内容。
