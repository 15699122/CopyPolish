"""预发布 tag 语义与版本同步脚本的单元测试。

运行方式：

    python3 -m unittest discover -s tests -v

覆盖 v0.7.0-pre1 发布准备新增的门禁：

- ``check_version.py`` 对预发布后缀的处理（接受基础版本或完整预发布版本）；
- release.yml publish job 中的严格 tag 正则，必须与脚本的 tag 语义一致：
  既接受 ``vX.Y.Z`` 与 ``vX.Y.Z-preN``，也拒绝 ``v1x2y3``、``v1.2.3-`` 之类
  的非法 tag（旧 shell 通配符会误接受这些输入）。
"""

from __future__ import annotations

import json
import re
import shutil
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

REPO_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import check_version  # noqa: E402

WORKFLOW = REPO_ROOT / ".github" / "workflows" / "release.yml"

# 与 .github/workflows/release.yml publish job 的 Guard 步骤保持一致。
STRICT_TAG_RE = re.compile(r"^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$")

# 预发布判定：去掉 v 前缀后，基础版本之后仍带预发布后缀。
PRERELEASE_RE = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+-")

VERSION_FILES = (
    "frontend/package.json",
    "frontend/package-lock.json",
    "src-tauri/tauri.conf.json",
    "src-tauri/Cargo.toml",
)


def write_versions(root: Path, version: str) -> None:
    (root / "frontend/package.json").write_text(
        json.dumps({"version": version}), encoding="utf-8"
    )
    (root / "frontend/package-lock.json").write_text(
        json.dumps({"version": version}), encoding="utf-8"
    )
    (root / "src-tauri/tauri.conf.json").write_text(
        json.dumps({"version": version}), encoding="utf-8"
    )
    (root / "src-tauri/Cargo.toml").write_text(
        f'[package]\nname = "x"\nversion = "{version}"\n', encoding="utf-8"
    )


def run_check_version(root: Path, *args: str) -> int:
    argv = sys.argv
    sys.argv = ["check_version.py", *args]
    try:
        with mock.patch.object(check_version, "ROOT", root):
            return check_version.main()
    finally:
        sys.argv = argv


class StrictTagGuardTest(unittest.TestCase):
    """publish job 的 tag 守卫必须严格，且能区分正式版与预发布版。"""

    def test_accepts_stable_and_prerelease_tags(self) -> None:
        for tag in ("v0.7.0", "v0.7.0-pre1", "v1.2.3-pre.4", "v10.20.30-rc.1"):
            with self.subTest(tag=tag):
                self.assertRegex(tag, STRICT_TAG_RE)

    def test_rejects_malformed_tags(self) -> None:
        # 这些输入会被旧的 v[0-9]*.[0-9]*.[0-9]* shell 通配符误接受。
        for tag in ("v1x2y3", "v1.2.3-", "v1.2", "1.2.3", "v1.2.3 extra", ""):
            with self.subTest(tag=tag):
                self.assertNotRegex(tag, STRICT_TAG_RE)

    def test_prerelease_classification(self) -> None:
        self.assertIsNone(PRERELEASE_RE.match("0.7.0"))
        for version in ("0.7.0-pre1", "0.7.0-pre.4", "0.7.0-rc.1"):
            with self.subTest(version=version):
                self.assertIsNotNone(PRERELEASE_RE.match(version))

    def test_workflow_uses_strict_regex_and_prerelease_flags(self) -> None:
        """workflow 必须使用严格正则，并区分 --latest 与 --prerelease。"""
        text = WORKFLOW.read_text(encoding="utf-8")
        self.assertIn(
            r"^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$", text, "缺少严格 tag 正则"
        )
        # 旧的宽松 shell 通配符必须已从可执行代码中移除。
        # 注释里为了说明历史问题仍会引用该字符串，因此只检查非注释行。
        code_lines = [
            line
            for line in text.splitlines()
            if not line.lstrip().startswith("#")
        ]
        code = "\n".join(code_lines)
        self.assertNotIn(
            "v[0-9]*.[0-9]*.[0-9]*", code, "可执行代码中仍残留宽松 shell 通配符"
        )
        self.assertIn("--prerelease", text, "缺少预发布标记")
        self.assertIn("--latest=false", text, "预发布未显式声明不占用 latest")
        self.assertIn("inputs.expected_sha == ''", text, "缺少 expected_sha 必填守卫")


class CheckVersionPrereleaseTest(unittest.TestCase):
    """check_version.py 必须把预发布 tag 视为合法候选版本。"""

    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="copypolish-version-"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        for rel in VERSION_FILES:
            target = self.tmp / rel
            target.parent.mkdir(parents=True, exist_ok=True)

    def test_accepts_base_version_for_prerelease_tag(self) -> None:
        """源码基线为基础版本 0.7.0 时，check_version 应接受 v0.7.0-pre1。"""
        write_versions(self.tmp, "0.7.0")
        self.assertEqual(run_check_version(self.tmp, "v0.7.0-pre1"), 0)

    def test_accepts_full_prerelease_version(self) -> None:
        """prepare_release_version.py 写入的完整预发布版本同样应被接受。"""
        write_versions(self.tmp, "0.7.0-pre1")
        self.assertEqual(run_check_version(self.tmp, "v0.7.0-pre1"), 0)

    def test_rejects_mismatched_prerelease(self) -> None:
        """版本既不是基础版本也不是完整预发布版本时必须失败。"""
        write_versions(self.tmp, "0.6.2")
        self.assertEqual(run_check_version(self.tmp, "v0.7.0-pre1"), 1)

    def test_rejects_inconsistent_files(self) -> None:
        write_versions(self.tmp, "0.7.0")
        (self.tmp / "src-tauri/tauri.conf.json").write_text(
            json.dumps({"version": "0.6.2"}), encoding="utf-8"
        )
        self.assertEqual(run_check_version(self.tmp, "v0.7.0-pre1"), 1)


if __name__ == "__main__":
    unittest.main()