# 密钥管理（age + sops）

本工程中的敏感配置项（如 GitLab 部署 token 等）使用 [sops](https://github.com/getsops/sops) 与 [age](https://age-encryption.org/) 管理，而非明文或临时拼凑的 gpg 方案。

- `secrets/tokens.env` — **sops 加密**的 env 文件（值形如 `ENC[AES256_GCM,...]`）。
- `scripts/load_tokens.sh` — 检查 `sops`、解密并将变量注入当前 shell，不把明文写入磁盘。

## 当前落地状态

仓库只提交**示例模板**，真实文件仅保存在维护者本地（与 `.gitignore` 和 `scripts/security_check.py` 的追踪检查一致）：

- `.sops.yaml.example` 已提交；本地的 `.sops.yaml`（仅包含 age 公钥接收者）不得提交；
- `secrets/tokens.env.example` 已提交；本地加密的 `secrets/tokens.env`（业务变量值保持 SOPS 加密）不得提交；
- `scripts/load_tokens.sh` 已提交并保持可执行权限（`755`）；
- 个人配置仓库不再持有本项目的这些凭据文件。

本地加密文件包含 GitLab 运维所需变量（变量名可见，变量值不可见）：
`GITLAB_DEPLOY_TOKEN`、`GITLAB_PAT`、`GITLAB_PROJECT_TOKEN`、`GITLAB_DEPLOY_USER`。

使用前需要本机安装 `sops`，并持有与 `.sops.yaml` 接收者匹配的 age 私钥。CI 内置的 `CI_JOB_TOKEN`、GitLab MCP OAuth 凭据和 GitHub Release 凭据不写入该文件。

- `.sops.yaml` — 本地配置；声明 age 接收者（`age1...` **公钥**）以及需要加密的文件；从 `.sops.yaml.example` 复制，**不得提交**。

## 工作原理

- 持有 **age 私钥**的人即可解密文件。私钥**绝不提交**，仅存放在 `~/.config/sops/age/keys.txt`（权限 `600`）。
- `.sops.yaml` 里的公钥可安全提交；仅有公钥无法还原任何明文。

## 编辑某个密钥

```bash
sops secrets/tokens.env            # 用 $EDITOR 打开，保存时自动重新加密
sops --set '["GITLAB_PAT"] "new"' secrets/tokens.env   # 非交互地设置字段
```

## 从明文重新加密

```bash
# 先把明文放入 secrets/tokens.env（临时），然后：
sops -e -i secrets/tokens.env      # 切勿提交明文状态
```

## 令牌轮换 Runbook

真实令牌轮换分为“平台侧吊销/创建”和“仓库侧重新加密”两步，不能只修改 SOPS 文件而跳过平台操作：

1. 在 GitLab 中吊销即将替换的 PAT 或 Deploy Token，并创建满足最小权限和有效期要求的新 token；
2. 使用 `sops secrets/tokens.env` 或 `sops --set` 更新对应字段，保存后确认文件仍为 SOPS 加密格式；
3. 不把新 token 放入命令参数、终端输出、临时明文文件、remote URL 或日志；
4. 运行 `python3 scripts/security_check.py --require-sops`，只确认结构和加密状态，不解密输出值；
5. 使用 `source scripts/load_tokens.sh` 后仅检查变量非空，完成需要凭据的最小范围操作；
6. 轮换完成后再次确认旧 token 在 GitLab 平台已吊销，并保留本次提交的锁定范围和验证记录。

仓库无法验证 GitLab 平台上的 token 是否已吊销，因此“真实 token 已吊销”必须由维护者在 GitLab 审计记录中确认；本地脚本不得将平台状态推断为成功。

## 注入当前 shell 会话

```bash
source scripts/load_tokens.sh
```

脚本会把 `sops --decrypt` 的结果暂存在当前 shell 的变量中，解密成功后才执行 `eval`，随后清理临时变量；不会把明文写入文件。sops 依序从 `$SOPS_AGE_KEY`、`$AGE_KEY`、`~/.config/sops/age/keys.txt` 查找 age 私钥。

可先检查文件状态，不显示密钥值：

```bash
sops filestatus secrets/tokens.env
```

加载后只应验证变量是否存在，禁止打印变量值或将其写入日志：

```bash
source scripts/load_tokens.sh
test -n "${GITLAB_PAT:-}"
```

## 应急恢复（用备份私钥恢复）

若主私钥 `~/.config/sops/age/keys.txt` 丢失，可用备份私钥恢复解密能力。备份私钥就是一个普通的 `keys.txt` 文件，其存放位置由你自行决定（此处简记为 `/path/to/backup/key`），校验方式与普通密钥副本相同。

```bash
# 1.（可选）校验备份是有效的 age 密钥
age-keygen -y /path/to/backup/key          # 应回显对应的公钥

# 2. 将备份恢复到 sops 的默认查找位置
mkdir -p ~/.config/sops/age
cp /path/to/backup/key ~/.config/sops/age/keys.txt
chmod 600 ~/.config/sops/age/keys.txt

# 3. 确认解密可用
sops --decrypt secrets/tokens.env | grep -c '^export '   # >0 说明密钥已恢复

# 4. 照常注入当前 shell
source scripts/load_tokens.sh
```

也可以不改动 `HOME`，仅对当前会话临时指向备份密钥：

```bash
SOPS_AGE_KEY="$(grep -E '^AGE-SECRET-KEY' /path/to/backup/key)" \
  sops --decrypt secrets/tokens.env
```

备份是单一、自包含的文件——无需其它任何状态即可解锁加密密钥。

### 离线恢复演练记录

已完成一次离线恢复演练。演练在临时目录中生成临时 age 密钥和测试用 SOPS 文件，随后仅通过临时
`SOPS_AGE_KEY` 指向备份私钥，执行 `sops --decrypt` 并验证导出的测试变量数量；没有读取、打印或写回
生产 `secrets/tokens.env`，临时私钥和明文文件均在演练结束后删除。结果确认：备份私钥是自包含的，
不依赖原 `HOME` 或其它本机状态即可恢复解密能力。

## 安全约定

切勿提交令牌、凭据、明文密钥副本（`.plain`、`.dec`）、会话、日志、缓存或数据库。切勿提交 age **私钥**（`~/.config/sops/age/keys.txt`）。

- 只提交 SOPS 加密后的 `secrets/tokens.env`，不提交临时明文副本；
- 不将 PAT 写入 remote URL、命令参数、脚本、Release Notes 或构建日志；
- 令牌轮换后必须重新加密文件、验证当前私钥可解密，并及时吊销旧令牌；
- `CI_JOB_TOKEN` 只由 GitLab CI 在 job 内提供，不复制到长期凭据文件；
- 个人配置仓库仅保留迁移历史，不再作为本项目凭据的运行时来源。

## 自动检查门禁

提交前或发布前可运行：

```bash
python3 scripts/security_check.py
python3 scripts/security_check.py --require-sops
```

检查内容：

- Git 跟踪文件中的高置信度 GitLab/GitHub/AWS token、私钥和明文凭据赋值；
- `secrets/tokens.env` 的 SOPS 版本、MAC、age recipient 和加密值结构；
- 本机安装 sops 时，额外执行 `sops filestatus` 并确认文件处于加密状态。

脚本不会解密文件，也不会输出匹配到的凭据内容。GitLab tag pipeline 的
`security:check` job 使用结构校验和明文扫描作为构建前门禁；该 job 仅随合法
`v*` tag pipeline 执行，普通开发分支仍应在提交前本地运行检查。维护者本地应优先
使用 `--require-sops` 执行更强校验。

### 凭据暴露后的处理

若 token 曾出现在进程参数、终端输出、日志或第三方工具记录中，应立即将其视为
泄露：先在 GitLab 中吊销/轮换，再用新的加密值更新 `secrets/tokens.env`，最后运行
`python3 scripts/security_check.py --require-sops` 验证加密文件状态。不得用扫描脚本
替代 token 吊销或轮换操作。
