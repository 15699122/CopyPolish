#!/usr/bin/env bash
# Linux 本地构建与发布资产整理（roadmap §3 / docs/release/manual-release.md）。
#
# 约束：
# - 必须在干净的发布工作区运行（git status 无任何改动）；
#   推荐 git worktree 隔离发布目录，禁止在待提交的日常开发工作区直接执行；
# - 不创建 tag、不推送、不上传 Release；产物写入被忽略的 dist/；
# - Linux 资产必须在 Linux 上构建（本项目不配置交叉编译）。
#
# 用法：
#   ./scripts/build_release_local.sh <tag> [--skip-verify]
#
# 示例：
#   ./scripts/build_release_local.sh v0.5.0

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_DIR="$REPO_ROOT/dist"

usage() {
    echo "用法: $0 <vX.Y.Z[-suffix]> [--skip-verify]" >&2
    exit 2
}

[[ $# -ge 1 ]] || usage
TAG="$1"
shift || true
SKIP_VERIFY=0
for arg in "$@"; do
    case "$arg" in
        --skip-verify) SKIP_VERIFY=1 ;;
        *) usage ;;
    esac
done

echo "== 检查干净发布工作区 =="
if [[ -n "$(git -C "$REPO_ROOT" status --porcelain)" ]]; then
    echo "错误：工作区存在未提交改动。请在独立的发布 worktree 中执行。" >&2
    echo "参考 docs/release/manual-release.md 第 3 节。" >&2
    exit 1
fi

echo "== 同步 tag 完整版本 ($TAG) =="
python3 "$REPO_ROOT/scripts/prepare_release_version.py" "$TAG"

if [[ "$SKIP_VERIFY" -ne 1 ]]; then
    echo "== 发布前统一验证（与 CI 对齐）=="
    python3 "$REPO_ROOT/scripts/verify.py" --profile release --tag "$TAG"
else
    echo "== 跳过验证（--skip-verify）=="
fi

echo "== 构建 Linux bundle =="
npm ci --prefix "$REPO_ROOT/frontend"
npm run tauri --prefix "$REPO_ROOT/frontend" -- build

echo "== 构建并打包 Linux TUI 独立资产 =="
cargo build \
    --manifest-path "$REPO_ROOT/src-tauri/Cargo.toml" \
    --features tui \
    --release \
    --bin copypolish-tui

# TUI 独立资产：staging 目录内部压缩，根目录直接包含二进制。
TUI_STAGING="$DIST_DIR/tui-staging"
mkdir -p "$TUI_STAGING"
cp "$REPO_ROOT/src-tauri/target/release/copypolish-tui" "$TUI_STAGING/"
(cd "$TUI_STAGING" && 7z a -t7z -mx=9 "$DIST_DIR/CopyPolish-tui-linux-x86_64.7z" copypolish-tui)
rm -rf "$TUI_STAGING"

BUNDLE_DIR="$REPO_ROOT/src-tauri/target/release/bundle"
mkdir -p "$DIST_DIR"

echo "== 收集并统一命名资产 =="
find "$BUNDLE_DIR" -name '*.deb' -exec cp {} "$DIST_DIR/CopyPolish_linux_amd64.deb" \;
find "$BUNDLE_DIR" -name '*.rpm' -exec cp {} "$DIST_DIR/CopyPolish-linux-x86_64.rpm" \;
find "$BUNDLE_DIR" -name '*.AppImage' -exec cp {} "$DIST_DIR/CopyPolish_linux_amd64.AppImage" \;

echo "== 校验产物（Linux 平台资产）=="
python3 "$REPO_ROOT/scripts/verify_release_assets.py" "$TAG" \
    --dist-dir "$DIST_DIR" \
    --platform linux
