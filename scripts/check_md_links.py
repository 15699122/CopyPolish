#!/usr/bin/env python3
"""检查仓库内 Markdown 文件的相对链接是否指向存在的文件。

- 跳过外部链接（http/https/mailto）、纯锚点和围栏代码块内的内容；
- 跳过 .git / node_modules / target / dist 目录；
- 发现死链时以退出码 1 失败，供本地与 CI 门禁使用。
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SKIP_DIRS = {".git", "node_modules", "target", "dist"}
LINK_RE = re.compile(r"\[[^\]]*\]\(([^)\s]+)")


def markdown_files() -> list[Path]:
    return [
        path
        for path in ROOT.rglob("*.md")
        if not any(part in SKIP_DIRS for part in path.relative_to(ROOT).parts)
    ]


def strip_fenced_code(text: str) -> str:
    return re.sub(r"```.*?```", "", text, flags=re.S)


def main() -> int:
    broken: list[str] = []
    for path in markdown_files():
        text = strip_fenced_code(path.read_text(encoding="utf-8"))
        for match in LINK_RE.finditer(text):
            target = match.group(1).strip()
            if target.startswith(("#", "http://", "https://", "mailto:")):
                continue
            target = target.split("#", 1)[0]
            if not target:
                continue
            if not (path.parent / target).resolve().exists():
                broken.append(f"{path.relative_to(ROOT)} -> {target}")

    for entry in broken:
        print(f"broken link: {entry}")
    print(f"markdown link check: {len(broken)} broken")
    return 1 if broken else 0


if __name__ == "__main__":
    raise SystemExit(main())