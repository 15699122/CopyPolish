#!/usr/bin/env python3
"""Create the GitLab Release for the current CI tag.

The eight files (seven release assets plus SHA256SUMS) must already exist
in the Generic Package Registry.
Authentication is provided by CI_JOB_TOKEN.
"""

from __future__ import annotations

import json
import os
import urllib.request


def main() -> int:
    api = os.environ["CI_API_V4_URL"]
    project_id = os.environ["CI_PROJECT_ID"]
    tag = os.environ["CI_COMMIT_TAG"]
    commit_sha = os.environ["CI_COMMIT_SHA"]
    job_token = os.environ["CI_JOB_TOKEN"]
    package_url = (
        f"{api}/projects/{project_id}/packages/generic/copypolish/{tag}"
    )
    files = [
        "CopyPolish.exe",
        "CopyPolish-windows-x64.7z",
        "CopyPolish_linux_amd64.deb",
        "CopyPolish-linux-x86_64.rpm",
        "CopyPolish_linux_amd64.AppImage",
        "CopyPolish-tui-windows-x64.7z",
        "CopyPolish-tui-linux-x86_64.7z",
        "SHA256SUMS",
    ]
    payload = {
        "tag_name": tag,
        "ref": commit_sha,
        "name": tag,
        "prerelease": "-" in tag,
        "description": (
            f"# {tag}\n\n"
            f"- Commit: `{commit_sha}`\n"
            "- 七个发布资产已完成校验。\n"
            "- SHA-256 摘要见 `SHA256SUMS`。\n\n"
            "> 本 Release 由 GitLab CI 生成，正式对外发布前仍需人工复核 Release Notes。"
        ),
        "assets": {
            "links": [
                {"name": filename, "url": f"{package_url}/{filename}", "link_type": "package"}
                for filename in files
            ]
        },
    }
    request = urllib.request.Request(
        f"{api}/projects/{project_id}/releases",
        data=json.dumps(payload).encode("utf-8"),
        headers={
            "JOB-TOKEN": job_token,
            "Content-Type": "application/json",
        },
        method="POST",
    )
    with urllib.request.urlopen(request) as response:
        release = json.loads(response.read())
    print(
        f"OK: GitLab Release {release.get('tag_name')} created, "
        f"prerelease={release.get('prerelease')}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())