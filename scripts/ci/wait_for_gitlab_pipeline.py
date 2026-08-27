#!/usr/bin/env python3
"""Wait for the GitLab build-only pipeline matching a GitHub tag SHA."""

from __future__ import annotations

import json
import os
import sys
import time
import urllib.parse
import urllib.request


def get_json(url: str, token: str) -> object:
    request = urllib.request.Request(url, headers={"PRIVATE-TOKEN": token})
    with urllib.request.urlopen(request, timeout=30) as response:
        return json.loads(response.read())


def main() -> int:
    tag = sys.argv[1] if len(sys.argv) > 1 else os.environ["RELEASE_TAG"]
    expected_sha = os.environ["GITHUB_TAG_SHA"]
    project = urllib.parse.quote(os.environ.get("GITLAB_PROJECT_ID", "85804438"), safe="")
    api = os.environ.get("GITLAB_API_V4_URL", "https://gitlab.com/api/v4")
    token = os.environ["GITLAB_RELEASE_BRIDGE_TOKEN"]
    timeout = int(os.environ.get("GITLAB_PIPELINE_TIMEOUT_SECONDS", "5400"))
    deadline = time.time() + timeout

    while time.time() < deadline:
        pipelines = get_json(
            f"{api}/projects/{project}/pipelines?ref={urllib.parse.quote(tag)}&per_page=20",
            token,
        )
        if isinstance(pipelines, list):
            matches = [p for p in pipelines if p.get("sha") == expected_sha]
            if matches:
                pipeline = matches[0]
                status = pipeline.get("status")
                print(
                    f"GitLab pipeline id={pipeline.get('id')} "
                    f"status={status} sha={expected_sha}",
                    flush=True,
                )
                if status == "success":
                    return 0
                if status in {"failed", "canceled", "skipped", "manual"}:
                    print(f"GitLab pipeline finished unsuccessfully: {status}", file=sys.stderr)
                    return 1
        print("waiting for GitLab pipeline...", flush=True)
        time.sleep(20)

    print(f"timed out waiting for GitLab pipeline: {tag}", file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())