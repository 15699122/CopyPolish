#!/usr/bin/env python3
"""CopyPolish 的统一验证入口。

所有本地、GitHub Actions 和 GitLab 的发布前门禁都通过本脚本选择 profile，
避免在多个 YAML、Shell、PowerShell 和 Markdown 示例中重复维护命令。

用法：
    python3 scripts/verify.py --profile rust
    python3 scripts/verify.py --profile frontend
    python3 scripts/verify.py --profile checks
    python3 scripts/verify.py --profile security
    python3 scripts/verify.py --profile audit
    python3 scripts/verify.py --profile ci
    python3 scripts/verify.py --profile release --tag vX.Y.Z[-suffix]
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "src-tauri" / "Cargo.toml"
E2E_AUDIT_POLICY = ROOT / "docs" / "decisions" / "e2e-audit-policy.json"


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


def run_rust_audit() -> None:
    print("== Rust dependency audit ==", flush=True)
    result = subprocess.run(
        ["cargo", "audit", "--file", str(ROOT / "src-tauri" / "Cargo.lock"), "--json"],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    if result.returncode != 0 and not result.stdout.strip():
        if result.stderr:
            print(result.stderr, file=sys.stderr, end="")
        raise subprocess.CalledProcessError(result.returncode, result.args)
    try:
        report = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        print(result.stdout, end="")
        print(f"Rust dependency audit returned invalid JSON: {error}", file=sys.stderr)
        raise subprocess.CalledProcessError(result.returncode or 1, result.args) from error

    vulnerabilities = report.get("vulnerabilities", {})
    warnings = report.get("warnings", {})
    vulnerability_count = vulnerabilities.get("count", 0)
    warning_count = sum(len(items) for items in warnings.values())
    print(f"RustSec vulnerabilities: {vulnerability_count}")
    print(f"Allowed RustSec warnings: {warning_count}")
    if vulnerability_count:
        print("Rust dependency audit found security vulnerabilities", file=sys.stderr)
        raise subprocess.CalledProcessError(1, result.args)


def load_e2e_audit_policy() -> dict[str, dict[str, object]]:
    try:
        policy = json.loads(E2E_AUDIT_POLICY.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        print(f"E2E audit policy is unavailable or invalid: {error}", file=sys.stderr)
        raise subprocess.CalledProcessError(1, ["read", str(E2E_AUDIT_POLICY)]) from error
    if not isinstance(policy, dict) or not isinstance(policy.get("accepted_advisories"), list):
        print("E2E audit policy must contain an accepted_advisories list", file=sys.stderr)
        raise subprocess.CalledProcessError(1, ["read", str(E2E_AUDIT_POLICY)])
    return {
        item["id"]: item
        for item in policy["accepted_advisories"]
        if isinstance(item, dict) and isinstance(item.get("id"), str)
    }


def advisory_ids(vulnerability: dict[str, object]) -> set[str]:
    ids: set[str] = set()
    for item in vulnerability.get("via", []):
        if isinstance(item, dict) and isinstance(item.get("url"), str):
            url = item["url"]
            marker = "/advisories/"
            if marker in url:
                ids.add(url.split(marker, 1)[1].split("/", 1)[0])
    return ids


def run_npm_audit(label: str, prefix: Path, policy: dict[str, dict[str, object]] | None = None) -> None:
    print(f"== {label} ==", flush=True)
    result = subprocess.run(
        [
            "npm",
            "audit",
            "--prefix",
            str(prefix),
            "--audit-level=high",
            "--omit=optional",
            "--ignore-scripts",
            "--json",
        ],
        cwd=ROOT,
        check=False,
        capture_output=True,
        text=True,
    )
    try:
        report = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        if result.stderr:
            print(result.stderr, file=sys.stderr, end="")
        print(f"{label} returned invalid JSON; network/tool failure is not an audit pass: {error}", file=sys.stderr)
        raise subprocess.CalledProcessError(result.returncode or 1, result.args) from error

    vulnerabilities = report.get("vulnerabilities", {})
    if not isinstance(vulnerabilities, dict):
        raise subprocess.CalledProcessError(1, result.args)
    high_or_critical: dict[str, dict[str, object]] = {
        name: value
        for name, value in vulnerabilities.items()
        if isinstance(value, dict) and value.get("severity") in {"high", "critical"}
    }
    if not high_or_critical:
        print(f"{label}: no high/critical vulnerabilities")
        return
    if policy is None:
        print(f"{label}: {len(high_or_critical)} high/critical vulnerabilities", file=sys.stderr)
        raise subprocess.CalledProcessError(1, result.args)

    # npm audit 只在根 vulnerability（如 extract-zip）中包含 advisory URL，
    # 传播到 @wdio/* 的条目通常只包含依赖名。因此先从完整报告收集根 advisory，
    # 再确认所有 high/critical 项都由已登记的根 advisory 覆盖。
    report_ids = {
        advisory_id
        for vulnerability in vulnerabilities.values()
        if isinstance(vulnerability, dict)
        for advisory_id in advisory_ids(vulnerability)
    }
    accepted = report_ids & policy.keys()
    unresolved: list[str] = []
    if not accepted:
        unresolved.append(
            "no accepted advisory id found in report (high/critical entries: "
            + ", ".join(sorted(high_or_critical))
            + ")"
        )
    elif report_ids - policy.keys():
        unresolved.extend(f"unaccepted advisory {item}" for item in sorted(report_ids - policy.keys()))
    if unresolved:
        print("E2E audit has unaccepted high/critical vulnerabilities:", file=sys.stderr)
        for item in unresolved:
            print(f"  - {item}", file=sys.stderr)
        raise subprocess.CalledProcessError(1, result.args)
    print(f"{label}: {len(high_or_critical)} high/critical entries covered by {', '.join(sorted(accepted))}")


def audit_commands() -> list[tuple[str, list[str]]]:
    return [
        (
            "Frontend dependency audit",
            command(
                "npm",
                "audit",
                "--prefix",
                str(ROOT / "frontend"),
                "--audit-level=high",
                "--omit=optional",
                "--ignore-scripts",
            ),
        ),
    ]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--profile",
        required=True,
        choices=("rust", "frontend", "checks", "security", "audit", "ci", "release"),
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
    elif args.profile == "audit":
        missing = [tool for tool in ("cargo", "cargo-audit", "npm") if shutil.which(tool) is None]
        if missing:
            print("依赖审计工具缺失：" + ", ".join(missing) + "。请先安装工具后重试。", file=sys.stderr)
            return 1
        groups = [audit_commands()]
    elif args.profile == "ci":
        groups = [rust_commands(), frontend_commands(), checks_commands()]
    else:
        run("Release version check", command(sys.executable, "scripts/check_version.py", args.tag))
        groups = [frontend_commands(), rust_commands(), checks_commands()]

    try:
        if args.profile == "audit":
            run_rust_audit()
            run_npm_audit("Frontend dependency audit", ROOT / "frontend")
            run_npm_audit("E2E dependency audit", ROOT / "e2e", load_e2e_audit_policy())
            run("SBOM generation check", command(sys.executable, "scripts/generate_sbom.py", "--check"))
            run("License manifest check", command(sys.executable, "scripts/generate_licenses.py", "--check"))
            groups = []
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