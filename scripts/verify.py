#!/usr/bin/env python3
"""CopyPolish 的统一验证入口。

所有本地、GitHub Actions 和 GitLab 的发布前门禁都通过本脚本选择 profile，
避免在多个 YAML、Shell、PowerShell 和 Markdown 示例中重复维护命令。

用法：
    python3 scripts/verify.py --profile rust
    python3 scripts/verify.py --profile frontend
    python3 scripts/verify.py --profile checks
    python3 scripts/verify.py --profile security
    python3 scripts/verify.py --profile ci
    python3 scripts/verify.py --profile release --tag vX.Y.Z[-suffix]
"""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "src-tauri" / "Cargo.toml"


def command(*args: str) -> list[str]:
    return list(args)


def run(description: str, args: list[str]) -> None:
    print(f"== {description} ==", flush=True)
    subprocess.run(args, cwd=ROOT, check=True)


def rust_commands() -> list[tuple[str, list[str]]]:
    manifest = str(MANIFEST)
    return [
        ("Rust format check", command("cargo", "fmt", "--manifest-path", manifest, "--check")),
        (
            "Rust clippy (default)",
            command("cargo", "clippy", "--manifest-path", manifest, "--all-targets", "--", "-D", "warnings"),
        ),
        ("Rust test (default)", command("cargo", "test", "--manifest-path", manifest)),
        (
            "Rust clippy (tui)",
            command(
                "cargo",
                "clippy",
                "--manifest-path",
                manifest,
                "--features",
                "tui",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ),
        ),
        (
            "Rust test (tui)",
            command("cargo", "test", "--manifest-path", manifest, "--features", "tui"),
        ),
        (
            "TUI build",
            command(
                "cargo",
                "build",
                "--manifest-path",
                manifest,
                "--features",
                "tui",
                "--bin",
                "copypolish-tui",
            ),
        ),
        (
            "Performance gate",
            command(sys.executable, "scripts/check_performance.py"),
        ),
    ]


def frontend_commands() -> list[tuple[str, list[str]]]:
    return [
        ("Frontend dependencies", command("npm", "ci", "--prefix", str(ROOT / "frontend"))),
        (
            "Frontend tests",
            command("npm", "test", "--prefix", str(ROOT / "frontend"), "--", "--run"),
        ),
        ("Frontend build", command("npm", "run", "build", "--prefix", str(ROOT / "frontend"))),
    ]


def checks_commands() -> list[tuple[str, list[str]]]:
    python = sys.executable
    return [
        ("Git diff check", command("git", "diff", "--check")),
        ("Secret and SOPS checks", command(python, "scripts/security_check.py")),
        ("Markdown link check", command(python, "scripts/check_md_links.py")),
    ]


def security_commands() -> list[tuple[str, list[str]]]:
    return [("Secret and SOPS checks", command(sys.executable, "scripts/security_check.py"))]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--profile",
        required=True,
        choices=("rust", "frontend", "checks", "security", "ci", "release"),
        help="要执行的验证集合",
    )
    parser.add_argument("--tag", help="发布 tag；仅 release profile 使用")
    args = parser.parse_args()

    if args.tag and args.profile != "release":
        parser.error("--tag 只能与 --profile release 一起使用")
    if args.profile == "release" and not args.tag:
        parser.error("--profile release 必须提供 --tag")

    groups: list[list[tuple[str, list[str]]]]
    if args.profile == "rust":
        groups = [rust_commands()]
    elif args.profile == "frontend":
        groups = [frontend_commands()]
    elif args.profile == "checks":
        groups = [checks_commands()]
    elif args.profile == "security":
        groups = [security_commands()]
    elif args.profile == "ci":
        groups = [rust_commands(), frontend_commands(), checks_commands()]
    else:
        run("Release version check", command(sys.executable, "scripts/check_version.py", args.tag))
        groups = [frontend_commands(), rust_commands(), checks_commands()]

    try:
        for group in groups:
            for description, args_list in group:
                run(description, args_list)
    except subprocess.CalledProcessError as error:
        print(f"验证失败：命令退出码 {error.returncode}", file=sys.stderr)
        return error.returncode or 1

    print(f"验证完成：profile={args.profile}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())