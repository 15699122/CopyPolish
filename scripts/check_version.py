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
    cargo_version = m.group(1) if m else ""
    return {
        "frontend/package.json": pkg.get("version") or "",
        "frontend/package-lock.json": lock.get("version") or "",
        "src-tauri/tauri.conf.json": conf.get("version") or "",
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
        # 允许 semver 预发布后缀（如 0.4.0-pre.3）；版本号文件只保存数值部分。
        m = re.fullmatch(r"(\d+\.\d+\.\d+)(-[0-9A-Za-z.-]+)?", expected)
        if not m:
            print(f"ERROR: 非法 tag 版本号格式: {tag}")
            return 1
        base, suffix = m.group(1), m.group(2) or ""
        full = base + suffix
        for k, v in versions_found.items():
            # 源码状态允许为基础版本；release 工作流执行
            # prepare_release_version.py 后允许为完整预发布版本。
            if v != base and v != full:
                ok = False
                print(f"ERROR: {k} 版本 {v} 既不等于基础版本 {base} 也不等于完整版本 {full}")

    if ok:
        print(f"OK: 所有版本号一致 ({next(iter(distinct))})")
        return 0
    return 1


if __name__ == "__main__":
    raise SystemExit(main())