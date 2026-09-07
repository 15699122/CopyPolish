"""scripts/security_check.py 的防泄露回归测试。

运行方式：

    python3 -m unittest discover -s tests -v

核心目标（对应 CodeQL py/clear-text-logging-sensitive-data）：
扫描器只能输出仓库相对路径、行号和凭据类型标签，
任何凭据原文、匹配行内容或其派生字符串都不得进入
Finding、诊断文本或 stdout/stderr。
"""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import security_check  # noqa: E402

# 各模式的 canary 样本：足够长以命中模式，但不是真实凭据。
CANARIES = {
    "GitLab token": "glpat-" + "A1b2C3d4E5f6G7h8I9j0K1l2",
    "GitHub classic token": "ghp_" + "Zz9Yy8Xx7Ww6Vv5Uu4Tt3Ss2Rr1Qq0Pp",
    "GitHub fine-grained token": "github_pat_" + "Aa1Bb2Cc3Dd4Ee5Ff6Gg7",
    "AWS access key": "AKIA" + "IOSFODNN7EXAMPLE",
    "age private key": "AGE-SECRET-KEY-" + "1QQQQQQQQQQQQQQQQQQQQQQQQ",
    "private key block": "-----BEGIN " + "RSA " + "PRIVATE KEY-----",
    "plain GitLab credential assignment": "GITLAB_PAT=" + "supersecretvalue123",
}


class ClassifyLineTest(unittest.TestCase):
    def test_canary_lines_classified_by_label(self) -> None:
        for kind, sample in CANARIES.items():
            with self.subTest(kind=kind):
                self.assertEqual(security_check.classify_line(sample), kind)

    def test_benign_lines_return_none(self) -> None:
        for sample in (
            "GITLAB_PAT=ENC[AES256_GCM,data:placeholder]",
            "export GITHUB_TOKEN=${GITHUB_TOKEN}",
            "# see docs/security.md for token guidance",
            "const token = '<redacted>';",
            "",
        ):
            with self.subTest(sample=sample):
                self.assertIsNone(security_check.classify_line(sample))


class FindingRedactionTest(unittest.TestCase):
    def test_finding_never_carries_line_content(self) -> None:
        canary = CANARIES["GitHub classic token"]
        kind = security_check.classify_line(canary)
        self.assertIsNotNone(kind)
        finding = security_check.Finding(path="docs/example.md", line_number=3, kind=kind)
        rendered = ", ".join([finding.path, str(finding.line_number), finding.kind])
        self.assertNotIn(canary, rendered)
        self.assertNotIn(canary, str(finding))

    def test_format_findings_output_is_sanitized(self) -> None:
        findings = [
            security_check.Finding(path="a.md", line_number=1, kind="GitLab token"),
            security_check.Finding(path="b.md", line_number=2, kind="AWS access key"),
        ]
        formatted = security_check.format_findings(findings)
        self.assertEqual(formatted, ["a.md:1: GitLab token", "b.md:2: AWS access key"])
        for sample in CANARIES.values():
            for line in formatted:
                self.assertNotIn(sample, line)


class ScanPlaintextSecretsTest(unittest.TestCase):
    def test_scan_reports_sanitized_finding_for_canary(self) -> None:
        canary = CANARIES["GitHub classic token"]
        with tempfile.TemporaryDirectory() as tmp:
            worktree = Path(tmp) / "repo"
            worktree.mkdir()
            target = worktree / "leak-canary.md"
            target.write_text(f"token: {canary}\n", encoding="utf-8")
            git = subprocess.run(
                ["git", "init", "-q", str(worktree)],
                capture_output=True,
                text=True,
            )
            self.assertEqual(git.returncode, 0, git.stderr)
            staged = subprocess.run(
                ["git", "add", "leak-canary.md"],
                cwd=worktree,
                capture_output=True,
                text=True,
            )
            self.assertEqual(staged.returncode, 0, staged.stderr)

            original_root = security_check.REPO_ROOT
            try:
                security_check.REPO_ROOT = worktree
                findings = security_check.scan_plaintext_secrets()
            finally:
                security_check.REPO_ROOT = original_root

        self.assertEqual(len(findings), 1)
        finding = findings[0]
        self.assertEqual(finding.path, "leak-canary.md")
        self.assertEqual(finding.line_number, 1)
        self.assertEqual(finding.kind, "GitHub classic token")
        # 绝不携带原文。
        self.assertNotIn(canary, str(finding))
        for line in security_check.format_findings(findings):
            self.assertNotIn(canary, line)

    def test_ignored_paths_are_skipped(self) -> None:
        self.assertIn("secrets/tokens.env", security_check.IGNORED_PATHS)
        self.assertIn(".git", security_check.IGNORED_PATHS)


class MainOutputTest(unittest.TestCase):
    def test_main_stderr_has_no_canary_when_scan_fails(self) -> None:
        canary = CANARIES["GitHub classic token"]
        script = REPO_ROOT / "scripts" / "security_check.py"
        with tempfile.TemporaryDirectory() as tmp:
            worktree = Path(tmp) / "repo"
            (worktree / "scripts").mkdir(parents=True)
            shutil.copy2(script, worktree / "scripts" / "security_check.py")
            (worktree / "leak-canary.md").write_text(f"token: {canary}\n", encoding="utf-8")
            subprocess.run(["git", "init", "-q", str(worktree)], capture_output=True)
            subprocess.run(
                ["git", "add", "leak-canary.md"],
                cwd=worktree,
                capture_output=True,
            )

            completed = subprocess.run(
                [sys.executable, str(worktree / "scripts" / "security_check.py")],
                cwd=worktree,
                capture_output=True,
                text=True,
            )

        self.assertNotEqual(completed.returncode, 0)
        combined = completed.stdout + completed.stderr
        self.assertIn("leak-canary.md:1: GitHub classic token", combined)
        self.assertNotIn(canary, combined)


class CLISmokeTest(unittest.TestCase):
    def test_repo_self_check_passes_without_secrets_file(self) -> None:
        """在真实仓库上运行 main()，仓库当前不应有明文凭据告警。"""
        script = REPO_ROOT / "scripts" / "security_check.py"
        completed = subprocess.run(
            [sys.executable, str(script)],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
        )
        self.assertEqual(completed.returncode, 0, completed.stderr)
        self.assertIn("OK: plaintext secret scan", completed.stdout)


if __name__ == "__main__":
    unittest.main()
