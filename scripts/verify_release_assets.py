#!/usr/bin/env python3
"""校验本地发布产物（roadmap §3）。

用法：
    python3 scripts/verify_release_assets.py <tag> [--dist-dir dist]

校验内容：
1. tag 命名符合 vX.Y.Z 或 vX.Y.Z-suffix；名称含 "-" 视为预发布；
2. 版本一致性（复用 scripts/check_version.py）；
3. 七个发布资产存在且命名正确（桌面版 Windows/Linux 五项 + TUI 独立资产两项）；
4. Windows .7z 在 staging 目录内部压缩：根目录直接包含 CopyPolish.exe，
   不允许出现额外的父目录层。

约束：本脚本只读校验，不创建 tag、不推送、不上传 Release。
"""

from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

WINDOWS_ASSETS = (
    "CopyPolish.exe",
    "CopyPolish-windows-x64.7z",
)

LINUX_ASSETS = (
    "CopyPolish_linux_amd64.deb",
    "CopyPolish-linux-x86_64.rpm",
    "CopyPolish_linux_amd64.AppImage",
)

# TUI 独立发布资产（决策：TUI 与桌面版共享 Release 与发布方式，命名规范一致）。
# 包内根目录直接包含对应二进制：Windows 为 CopyPolish-tui.exe，
# Linux 为 copypolish-tui。
TUI_ASSETS = (
    "CopyPolish-tui-windows-x64.7z",
    "CopyPolish-tui-linux-x86_64.7z",
)

EXPECTED_ASSETS = WINDOWS_ASSETS + LINUX_ASSETS + TUI_ASSETS

TAG_RE = re.compile(r"^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$")


def fail(errors: list[str], message: str) -> None:
    errors.append(message)


def check_tag(tag: str, errors: list[str]) -> bool:
    if not TAG_RE.match(tag):
        fail(errors, f"tag 命名不符合 vX.Y.Z[-suffix] 格式: {tag}")
        return False
    return True


def check_versions(tag: str, errors: list[str]) -> None:
    script = REPO_ROOT / "scripts" / "check_version.py"
    result = subprocess.run(
        [sys.executable, str(script), tag],
        cwd=REPO_ROOT,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        detail = (result.stdout + result.stderr).strip()
        fail(errors, f"版本一致性校验失败: {detail}")


def check_assets(
    dist_dir: Path, errors: list[str], platform: str
) -> list[Path]:
    expected = {
        "windows": WINDOWS_ASSETS + ("CopyPolish-tui-windows-x64.7z",),
        "linux": LINUX_ASSETS + ("CopyPolish-tui-linux-x86_64.7z",),
        "all": EXPECTED_ASSETS,
    }[platform]
    missing = [name for name in expected if not (dist_dir / name).is_file()]
    for name in missing:
        fail(errors, f"缺少发布资产: {dist_dir / name}")
    extras = [
        p.name
        for p in sorted(dist_dir.iterdir())
        if p.is_file() and p.name not in expected
    ]
    for name in extras:
        fail(errors, f"存在非预期文件（请清理后重试）: {name}")
    return [dist_dir / name for name in expected if not missing]


def check_7z_root_layout(archive: Path, errors: list[str]) -> None:
    # 桌面版包根目录必须含 CopyPolish.exe；TUI 包根目录必须含其平台二进制。
    required_binary = {
        "CopyPolish-windows-x64.7z": "CopyPolish.exe",
        "CopyPolish-tui-windows-x64.7z": "CopyPolish-tui.exe",
        "CopyPolish-tui-linux-x86_64.7z": "copypolish-tui",
    }.get(archive.name)
    if required_binary is None:
        return

    seven_zip = shutil.which("7z") or shutil.which("7za") or shutil.which("7zr")
    if seven_zip is None:
        # 没有 7z CLI 时跳过结构检查，但给出明确提示。
        print("WARN: 未找到 7z CLI，跳过 .7z 目录结构检查", file=sys.stderr)
        return

    result = subprocess.run(
        [seven_zip, "l", "-slt", str(archive)],
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="replace",
    )
    if result.returncode != 0:
        fail(errors, f".7z 列表读取失败: {archive} :: {(result.stderr or '').strip()}")
        return

    root_entries: set[str] = set()
    for line in result.stdout.splitlines():
        stripped = line.strip()
        if not stripped.startswith("Path = "):
            continue
        path = stripped[len("Path = ") :].strip().replace("\\", "/")
        if path.endswith("/"):
            path = path.rstrip("/")
        if "/" in path or not path:
            continue
        root_entries.add(path)

    if required_binary not in root_entries:
        fail(
            errors,
            f".7z 根目录未直接包含 {required_binary}——请在 staging 目录内部压缩，"
            "不要把 staging 目录本身压进包里",
        )
    dirs_at_root = [
        name
        for name in ("dist", "windows", "release", "staging")
        if any(entry == name for entry in root_entries)
    ]
    if dirs_at_root:
        fail(errors, f".7z 根目录出现可疑目录项: {', '.join(dirs_at_root)}")


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="校验本地发布产物")
    parser.add_argument("tag", help="发布 tag，如 v0.5.0 或 v0.5.1-pre1")
    parser.add_argument(
        "--dist-dir",
        default="dist",
        help="资产所在目录（默认 dist/）",
    )
    parser.add_argument(
        "--platform",
        choices=("windows", "linux", "all"),
        default="all",
        help="校验平台资产：windows、linux 或 all（默认 all）",
    )
    args = parser.parse_args(argv)

    errors: list[str] = []
    if check_tag(args.tag, errors):
        check_versions(args.tag, errors)

    dist_dir = (REPO_ROOT / args.dist_dir).resolve()
    if not dist_dir.is_dir():
        fail(errors, f"资产目录不存在: {dist_dir}")
    else:
        present = check_assets(dist_dir, errors, args.platform)
        platform_archives = {
            "windows": (
                "CopyPolish-windows-x64.7z",
                "CopyPolish-tui-windows-x64.7z",
            ),
            "linux": ("CopyPolish-tui-linux-x86_64.7z",),
            "all": (
                "CopyPolish-windows-x64.7z",
                "CopyPolish-tui-windows-x64.7z",
                "CopyPolish-tui-linux-x86_64.7z",
            ),
        }[args.platform]
        for name in platform_archives:
            archive = dist_dir / name
            if archive.is_file():
                check_7z_root_layout(archive, errors)
        if present and not errors:
            print(f"OK: {len(present)} 个资产命名齐全")

    if errors:
        print("FAIL: 发布产物校验未通过：", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1
    print("OK: 发布产物校验全部通过")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
