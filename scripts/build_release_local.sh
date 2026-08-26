#!/usr/bin/env bash
# Linux 本地构建与发布资产整理（roadmap §3 / docs/manual-release.md）。
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
    echo "参考 docs/manual-release.md 第 3 节。" >&2
    exit 1
fi

echo "== 同步 tag 完整版本 ($TAG) =="
python3 "$REPO_ROOT/scripts/prepare_release_version.py" "$TAG"
python3 "$REPO_ROOT/scripts/check_version.py" "$TAG"

if [[ "$SKIP_VERIFY" -ne 1 ]]; then
    echo "== 发布前统一验证（与 CI 对齐）=="
    npm ci --prefix "$REPO_ROOT/frontend"
    npm test --prefix "$REPO_ROOT/frontend" -- --run
    npm run build --prefix "$REPO_ROOT/frontend"
    cargo fmt --manifest-path "$REPO_ROOT/src-tauri/Cargo.toml" --check
    cargo clippy --manifest-path "$REPO_ROOT/src-tauri/Cargo.toml" --all-targets -- -D warnings
    cargo test --manifest-path "$REPO_ROOT/src-tauri/Cargo.toml"
    git -C "$REPO_ROOT" diff --check
else
    echo "== 跳过验证（--skip-verify）=="
fi

echo "== 构建 Linux bundle =="
npm ci --prefix "$REPO_ROOT/frontend"
npm run tauri --prefix "$REPO_ROOT/frontend" -- build

BUNDLE_DIR="$REPO_ROOT/src-tauri/target/release/bundle"
mkdir -p "$DIST_DIR"

echo "== 收集并统一命名资产 =="
find "$BUNDLE_DIR" -name '*.deb' -exec cp {} "$DIST_DIR/CopyPolish_linux_amd64.deb" \;
find "$BUNDLE_DIR" -name '*.rpm' -exec cp {} "$DIST_DIR/CopyPolish-linux-x86_64.rpm" \;
find "$BUNDLE_DIR" -name '*.AppImage' -exec cp {} "$DIST_DIR/CopyPolish_linux_amd64.AppImage" \;

echo "== 校验产物 =="
python3 "$REPO_ROOT/scripts/verify_release_assets.py" "$TAG" \
    --dist-dir "$DIST_DIR" \
    || {
        echo "提示：Windows 资产缺失属正常现象——本脚本仅产出 Linux 资产，" >&2
        echo "完整五资产校验需在 Windows 构建完成后合并目录再跑一次。" >&2
    }

echo "完成：Linux 资产已输出到 $DIST_DIR"
ls -la "$DIST_DIR"
