#!/usr/bin/env python3
"""CopyPolish 的本地生成目录安全清理入口。

只删除明确的白名单目录，不做任何宽泛的递归通配删除。
测试运行产物（e2e/artifacts/、e2e/settings-*/）遵循项目约定：
本地留存、结果记录到文档/CHANGELOG 后清理，不上传远程。

用法：
    python3 scripts/clean.py --dry-run     # 只打印将要删除的目录
    python3 scripts/clean.py --generated   # 清理构建缓存与测试产物（默认）
    python3 scripts/clean.py --deep        # 额外清理 node_modules（下次验证需重新 npm ci）
"""

from __future__ import annotations

import argparse
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# --generated：可再生构建缓存与测试运行产物（安全删除，构建会重新生成）。
GENERATED = [
    "frontend/dist",
    "src-tauri/gen",
    "src-tauri/target",
    "scripts/__pycache__",
    "e2e/artifacts",
]

# 测试临时设置目录前缀（e2e runner 使用，正常结束会自清理，异常时残留）。
SETTINGS_PREFIXES = ["e2e/settings-"]

# --deep：额外删除依赖目录；下次验证前需要重新 `npm ci`。
DEEP = [
    "frontend/node_modules",
    "e2e/node_modules",
]


def candidates(*, deep: bool) -> list[Path]:
    paths = [ROOT / rel for rel in GENERATED]
    paths.extend(
        path
        for path in sorted(ROOT.glob("e2e/settings-*"))
        if path.is_dir() and path.name.startswith(("settings-", "settings_"))
    )
    for prefix in SETTINGS_PREFIXES:
        paths.extend(sorted(ROOT.glob(f"{prefix}*")))
    if deep:
        paths.extend(ROOT / rel for rel in DEEP)
    # 去重并保持稳定顺序。
    seen: set[Path] = set()
    unique: list[Path] = []
    for path in paths:
        if path not in seen:
            seen.add(path)
            unique.append(path)
    return unique


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--generated", action="store_true", help="清理构建缓存与测试产物")
    mode.add_argument("--deep", action="store_true", help="额外清理 node_modules")
    parser.add_argument("--dry-run", action="store_true", help="只打印，不删除")
    args = parser.parse_args()

    targets = candidates(deep=args.deep)
    if not targets:
        print("没有需要清理的目录。")
        return 0

    for path in targets:
        relative = path.relative_to(ROOT)
        if not path.exists():
            continue
        if args.dry_run:
            print(f"[dry-run] 将删除 {relative}")
            continue
        shutil.rmtree(path)
        print(f"已删除 {relative}")

    if not args.dry_run:
        print("清理完成。")
    return 0


if __name__ == "__main__":
    sys.exit(main())
