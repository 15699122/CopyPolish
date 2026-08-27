#!/usr/bin/env bash
# Push the current GitHub release tag to the GitLab build-only repository.

set -euo pipefail

TAG="${1:?usage: $0 <tag>}"
GITLAB_REPOSITORY_URL="${GITLAB_REPOSITORY_URL:-https://gitlab.com/olivaceum-group/chinese_copywriting_formatter.git}"

[[ "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]] || {
    echo "invalid release tag: $TAG" >&2
    exit 1
}

[[ -n "${GITLAB_RELEASE_BRIDGE_TOKEN:-}" ]] || {
    echo "GITLAB_RELEASE_BRIDGE_TOKEN is required" >&2
    exit 1
}

LOCAL_SHA="$(git rev-parse "refs/tags/$TAG^{commit}")"
TEMP_HOME="$(mktemp -d)"
trap 'rm -rf "$TEMP_HOME"' EXIT
chmod 700 "$TEMP_HOME"

git -c credential.helper="store --file=$TEMP_HOME/credentials" \
    credential approve <<EOF
protocol=https
host=gitlab.com
path=olivaceum-group/chinese_copywriting_formatter.git
username=github-release-bridge
password=$GITLAB_RELEASE_BRIDGE_TOKEN
EOF

REMOTE_SHA="$(git -c credential.helper="store --file=$TEMP_HOME/credentials" \
    ls-remote "$GITLAB_REPOSITORY_URL" "refs/tags/$TAG" | awk '{print $1}')"

if [[ -n "$REMOTE_SHA" && "$REMOTE_SHA" != "$LOCAL_SHA" ]]; then
    echo "GitLab tag already exists with a different commit: $TAG" >&2
    echo "local=$LOCAL_SHA remote=$REMOTE_SHA" >&2
    exit 1
fi

if [[ "$REMOTE_SHA" == "$LOCAL_SHA" ]]; then
    echo "GitLab tag already matches: $TAG ($LOCAL_SHA)"
    exit 0
fi

git -c credential.helper="store --file=$TEMP_HOME/credentials" \
    push "$GITLAB_REPOSITORY_URL" "refs/tags/$TAG:refs/tags/$TAG"

echo "pushed GitHub tag $TAG to GitLab at $LOCAL_SHA"