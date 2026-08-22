#!/usr/bin/env python3
"""校验各配置文件中的版本号一致，且与 Git tag（vX.Y.Z）匹配。

用法：python3 scripts/check_version.py [vX.Y.Z]
省略参数时仅校验文件之间的一致性。
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def read_text(rel: str) -> str:
    return (ROOT / rel).read_text(encoding="utf-8")


def versions() -> dict[str, str]:
    pkg = json.loads(read_text("frontend/package.json"))
    lock = json.loads(read_text("frontend/package-lock.json"))
    conf = json.loads(read_text("src-tauri/tauri.conf.json"))
    cargo = read_text("src-tauri/Cargo.toml")
    m = re.search(r'(?m)^version\s*=\s*"([^"]+)"', cargo)
    cargo_version = m.group(1) if m else None
    return {
        "frontend/package.json": pkg.get("version"),
        "frontend/package-lock.json": lock.get("version"),
        "src-tauri/tauri.conf.json": conf.get("version"),
        "src-tauri/Cargo.toml": cargo_version,
    }


def main() -> int:
    versions_found = {k: v for k, v in versions().items()}
    missing = [k for k, v in versions_found.items() if not v]
    if missing:
        print(f"ERROR: 无法读取以下文件的版本号: {', '.join(missing)}")
        return 1

    distinct = set(versions_found.values())
    ok = True
    if len(distinct) != 1:
        ok = False
        print("ERROR: 版本号不一致：")
        for k, v in versions_found.items():
            print(f"  {k}: {v}")

    tag = sys.argv[1] if len(sys.argv) > 1 else ""
    if tag:
        expected = tag[1:] if tag.startswith("v") else tag
        if not re.fullmatch(r"\d+\.\d+\.\d+", expected):
            print(f"ERROR: 非法 tag 版本号格式: {tag}")
            return 1
        for k, v in versions_found.items():
            if v != expected:
                ok = False
                print(f"ERROR: {k} 版本 {v} != tag 版本 {expected}")

    if ok:
        print(f"OK: 所有版本号一致 ({next(iter(distinct))})")
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())