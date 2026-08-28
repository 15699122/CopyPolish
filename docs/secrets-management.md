# 密钥管理（age + sops）

本工程中的敏感配置项（如 GitLab 部署 token 等）使用 [sops](https://github.com/getsops/sops) 与 [age](https://age-encryption.org/) 管理，而非明文或临时拼凑的 gpg 方案。

- `secrets/tokens.env` — **sops 加密**的 env 文件（值形如 `ENC[AES256_GCM,...]`）。
- `.sops.yaml` — 配置；声明 age 接收者（`age1...` **公钥**）以及需要加密的文件。

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

## 注入当前 shell 会话

```bash
source scripts/load_tokens.sh
```

`eval "$(sops --decrypt ...)"` 会把变量注入当前 shell——不落盘任何内容。sops 依序从 `$SOPS_AGE_KEY`、`$AGE_KEY`、`~/.config/sops/age/keys.txt` 查找 age 私钥。

## 添加其它接收者（如同事 / CI）

```bash
sops add-keys --age age1...,age1other... secrets/tokens.env
sops --rotate-keys secrets/tokens.env   # 重新配钥（移除已删除的接收者）
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

## 安全约定

切勿提交令牌、凭据、明文密钥副本（`.plain`、`.dec`）、会话、日志、缓存或数据库。切勿提交 age **私钥**（`~/.config/sops/age/keys.txt`）。
