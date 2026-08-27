#!/usr/bin/env bash
# 将本地 Linux Release 资产上传到 GitLab Generic Package Registry。
# 用法：GITLAB_TOKEN=... ./scripts/upload_gitlab_linux_assets.sh <tag> [dist-dir]

set -euo pipefail

TAG="${1:?用法: GITLAB_TOKEN=... $0 <vX.Y.Z[-suffix]> [dist-dir]}"
DIST_DIR="${2:-dist}"
PROJECT_ID="${GITLAB_PROJECT_ID:-85804438}"
GITLAB_API="${GITLAB_API:-https://gitlab.com/api/v4}"

if [[ -z "${GITLAB_TOKEN:-}" ]]; then
    echo "错误：请通过环境变量 GITLAB_TOKEN 提供 PAT/Deploy Token。" >&2
    exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# 该脚本通常在 build_release_local.sh 已完成版本同步的隔离 worktree 中执行；
# 因而不能用 git status 判断 clean。调用方必须自行确保工作区不是日常开发目录。
python3 scripts/check_version.py "$TAG"
python3 scripts/verify_release_assets.py "$TAG" --dist-dir "$DIST_DIR" --platform linux

LOCAL_TAG_SHA="$(git rev-parse "$TAG^{commit}")"
REMOTE_TAG_SHA="$(git ls-remote "https://gitlab.com/olivaceum-group/chinese_copywriting_formatter.git" "refs/tags/$TAG" | awk '{print $1}')"
if [[ -z "$REMOTE_TAG_SHA" || "$LOCAL_TAG_SHA" != "$REMOTE_TAG_SHA" ]]; then
    echo "错误：本地 tag 与 GitLab tag 不一致。" >&2
    echo "  local : $LOCAL_TAG_SHA" >&2
    echo "  remote: ${REMOTE_TAG_SHA:-<missing>}" >&2
    exit 1
fi

PACKAGE_URL="$GITLAB_API/projects/$PROJECT_ID/packages/generic/copypolish/$TAG"
FILES=(
    CopyPolish_linux_amd64.deb
    CopyPolish-linux-x86_64.rpm
    CopyPolish_linux_amd64.AppImage
)

for file in "${FILES[@]}"; do
    path="$DIST_DIR/$file"
    [[ -s "$path" ]] || { echo "错误：资产为空或不存在: $path" >&2; exit 1; }
    url="$PACKAGE_URL/$file"
    status="$(curl -sS -o /dev/null -w '%{http_code}' \
        -H "PRIVATE-TOKEN: $GITLAB_TOKEN" "$url")"
    if [[ "$status" == "200" ]]; then
        tmp="$(mktemp)"
        curl -fsSL -H "PRIVATE-TOKEN: $GITLAB_TOKEN" "$url" -o "$tmp"
        if cmp -s "$path" "$tmp"; then
            echo "已存在且 SHA 相同，跳过: $file"
            rm -f "$tmp"
            continue
        fi
        rm -f "$tmp"
        echo "错误：远端同名资产 SHA 不同，拒绝覆盖: $file" >&2
        exit 1
    elif [[ "$status" != "404" ]]; then
        echo "错误：检查远端资产失败，HTTP $status: $file" >&2
        exit 1
    fi
    echo "上传: $file"
    curl --fail --show-error --location \
        --header "PRIVATE-TOKEN: $GITLAB_TOKEN" \
        --upload-file "$path" "$url"
done

echo "Linux 资产上传完成：$PACKAGE_URL"