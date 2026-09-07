#!/usr/bin/env python3
"""检查仓库中的明文凭据与 SOPS 文件结构。

该脚本不解密任何文件，也不会打印匹配到的原文。它适合在 GitLab CI
和本地提交前检查中运行：

    python3 scripts/security_check.py
    python3 scripts/security_check.py --require-sops

CI 镜像可以只依赖 Python 和 Git；若安装了 sops，脚本会额外执行
``sops filestatus``，使用 ``--require-sops`` 可把该检查提升为硬门禁。
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parent.parent
SOPS_FILE = REPO_ROOT / "secrets" / "tokens.env"
SOPS_EXAMPLE_FILE = REPO_ROOT / "secrets" / "tokens.env.example"

# 只匹配高置信度的凭据形态，避免把文档中的变量名或普通示例误报。
SECRET_PATTERNS: tuple[tuple[str, re.Pattern[str]], ...] = (
    ("GitLab token", re.compile(r"\bglpat-[A-Za-z0-9_-]{20,}")),
    ("GitHub classic token", re.compile(r"\bgh[pousr]_[A-Za-z0-9_]{20,}")),
    ("GitHub fine-grained token", re.compile(r"\bgithub_pat_[A-Za-z0-9_]{20,}")),
    ("AWS access key", re.compile(r"\bAKIA[0-9A-Z]{16}\b")),
    ("age private key", re.compile(r"AGE-SECRET-KEY-[0-9A-Z-]{20,}")),
    (
        "private key block",
        re.compile(r"-----BEGIN (?:RSA |EC |OPENSSH |DSA )?PRIVATE KEY-----"),
    ),
    (
        "plain GitLab credential assignment",
        re.compile(
            r"\b(?:GITLAB_PAT|GITLAB_PROJECT_TOKEN|GITLAB_DEPLOY_TOKEN)\s*=\s*"
            r"(?!ENC\[|\$\{|\"\"|''|<[^>]+>)[^\s#\"']{12,}"
        ),
    ),
)

IGNORED_PATHS = {
    ".git",
    "secrets/tokens.env",
}


@dataclass(frozen=True)
class Finding:
    """脱敏后的扫描发现。

    设计约束（对应 CodeQL py/clear-text-logging-sensitive-data 告警）：
    Finding 只允许包含仓库相对路径、行号和凭据类型标签，
    绝不保存被匹配的源行内容、凭据原文或其任何派生字符串。
    """

    path: str
    line_number: int
    kind: str


def classify_line(line: str) -> str | None:
    """仅判断该行是否匹配某个凭据模式。

    返回静态类型标签或不返回任何内容；被检查的行内容本身
    不会进入返回值，也不会被调用方保留。
    """
    for label, pattern in SECRET_PATTERNS:
        if pattern.search(line) is not None:
            return label
    return None


def run(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=REPO_ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        check=False,
    )


def tracked_files() -> list[Path]:
    result = run(["git", "ls-files", "-z"])
    if result.returncode != 0:
        raise RuntimeError(result.stderr.strip() or "git ls-files failed")
    return [REPO_ROOT / item for item in result.stdout.split("\0") if item]


def is_binary(data: bytes) -> bool:
    return b"\0" in data


def scan_plaintext_secrets() -> list[Finding]:
    findings: list[Finding] = []
    for path in tracked_files():
        relative = path.relative_to(REPO_ROOT).as_posix()
        if relative in IGNORED_PATHS or not path.is_file():
            continue
        data = path.read_bytes()
        if is_binary(data):
            continue
        text = data.decode("utf-8", errors="replace")
        for line_number, line in enumerate(text.splitlines(), start=1):
            kind = classify_line(line)
            if kind is not None:
                # 只保留脱敏字段；匹配行内容在此处被丢弃。
                findings.append(Finding(path=relative, line_number=line_number, kind=kind))
    return findings


def format_findings(findings: list[Finding]) -> list[str]:
    """将 Finding 渲染为诊断文本。

    输入只能是 Finding 结构，保证凭据原文不可能通过该路径
    进入 stdout/stderr。
    """
    return [f"{finding.path}:{finding.line_number}: {finding.kind}" for finding in findings]


def validate_sops_structure() -> list[str]:
    errors: list[str] = []
    if not SOPS_FILE.is_file():
        if not SOPS_EXAMPLE_FILE.is_file():
            return [f"missing SOPS template: {SOPS_EXAMPLE_FILE.relative_to(REPO_ROOT)}"]
        return []

    text = SOPS_FILE.read_text(encoding="utf-8", errors="replace")
    lines = text.splitlines()
    if "sops_version=" not in text:
        errors.append("secrets/tokens.env is missing sops_version metadata")
    if "sops_mac=" not in text:
        errors.append("secrets/tokens.env is missing sops_mac metadata")
    if "sops_age__list_0__map_recipient=" not in text:
        errors.append("secrets/tokens.env is missing an age recipient")
    if "-----BEGIN AGE ENCRYPTED FILE-----" not in text:
        errors.append("secrets/tokens.env is missing encrypted age metadata")

    expected_names = {
        "GITLAB_DEPLOY_TOKEN",
        "GITLAB_PAT",
        "GITLAB_PROJECT_TOKEN",
        "GITLAB_DEPLOY_USER",
    }
    names: set[str] = set()
    for line in lines:
        match = re.match(r"export ([A-Z0-9_]+)=([^#]*)", line)
        if match:
            names.add(match.group(1))
            value = match.group(2).strip()
            if not value.startswith("ENC["):
                errors.append(f"secrets/tokens.env contains a non-SOPS value for {match.group(1)}")
    missing = sorted(expected_names - names)
    if missing:
        errors.append(f"secrets/tokens.env is missing expected encrypted variables: {', '.join(missing)}")
    return errors


def validate_with_sops(require_sops: bool) -> list[str]:
    if not SOPS_FILE.is_file():
        return []
    sops = shutil.which("sops")
    if sops is None:
        if require_sops:
            return ["sops is required but was not found"]
        print("WARN: sops not found; structural SOPS validation only", file=sys.stderr)
        return []

    result = run([sops, "filestatus", str(SOPS_FILE)])
    if result.returncode != 0:
        return ["sops filestatus failed"]
    try:
        status = json.loads(result.stdout)
    except json.JSONDecodeError:
        return ["sops filestatus returned invalid JSON"]
    if status.get("encrypted") is not True:
        return ["sops filestatus did not report an encrypted file"]
    return []


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="检查明文凭据与 SOPS 文件结构")
    parser.add_argument(
        "--require-sops",
        action="store_true",
        help="要求本机安装 sops 并通过 sops filestatus 校验",
    )
    args = parser.parse_args(argv)

    errors = format_findings(scan_plaintext_secrets())
    tracked_secrets = run(["git", "ls-files", "--error-unmatch", "secrets/tokens.env"])
    if tracked_secrets.returncode == 0:
        errors.append("secrets/tokens.env must not be tracked; keep it local and use secrets/tokens.env.example")
    tracked_sops = run(["git", "ls-files", "--error-unmatch", ".sops.yaml"])
    if tracked_sops.returncode == 0:
        errors.append(".sops.yaml must not be tracked; use .sops.yaml.example")
    errors.extend(validate_sops_structure())
    errors.extend(validate_with_sops(args.require_sops))

    if errors:
        print("FAIL: security checks failed", file=sys.stderr)
        for error in errors:
            print(f"  - {error}", file=sys.stderr)
        return 1

    print("OK: plaintext secret scan and SOPS metadata checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())