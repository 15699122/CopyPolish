#!/usr/bin/env python3
"""Download GitLab Generic Package assets and verify SHA256SUMS."""

from __future__ import annotations

import hashlib
import os
import pathlib
import sys
import urllib.parse
import urllib.request


ASSETS = (
    "CopyPolish.exe",
    "CopyPolish-windows-x64.7z",
    "CopyPolish_linux_amd64.deb",
    "CopyPolish-linux-x86_64.rpm",
    "CopyPolish_linux_amd64.AppImage",
    "SHA256SUMS",
)


def main() -> int:
    tag = sys.argv[1] if len(sys.argv) > 1 else os.environ["RELEASE_TAG"]
    output = pathlib.Path(sys.argv[2] if len(sys.argv) > 2 else "release-files")
    project = urllib.parse.quote(os.environ.get("GITLAB_PROJECT_ID", "85804438"), safe="")
    api = os.environ.get("GITLAB_API_V4_URL", "https://gitlab.com/api/v4")
    token = os.environ["GITLAB_RELEASE_BRIDGE_TOKEN"]
    base = f"{api}/projects/{project}/packages/generic/copypolish/{urllib.parse.quote(tag)}"
    output.mkdir(parents=True, exist_ok=True)

    for filename in ASSETS:
        destination = output / filename
        request = urllib.request.Request(
            f"{base}/{urllib.parse.quote(filename)}",
            headers={"PRIVATE-TOKEN": token},
        )
        print(f"downloading {filename}", flush=True)
        with urllib.request.urlopen(request, timeout=180) as response:
            destination.write_bytes(response.read())

    for line in (output / "SHA256SUMS").read_text(encoding="utf-8").splitlines():
        digest, filename = line.split(maxsplit=1)
        actual = hashlib.sha256((output / filename).read_bytes()).hexdigest()
        if actual != digest:
            raise RuntimeError(f"SHA-256 mismatch for {filename}")

    print(f"verified {len(ASSETS) - 1} release assets")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (KeyError, OSError, RuntimeError, ValueError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        raise SystemExit(1)