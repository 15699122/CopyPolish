"""scripts/verify_release_assets.py 元数据校验的单元测试。

运行方式：

    python3 -m unittest discover -s tests -v

覆盖 Plan PR-R3 要求的正向/负向用例：SBOM 格式与版本一致性、
SHA256SUMS 覆盖完整性/重复/篡改/路径安全、资产 allowlist。
"""

from __future__ import annotations

import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO_ROOT / "scripts"))

import verify_release_assets as vra  # noqa: E402


def make_sbom(version: str = "0.6.2", components: int = 1) -> str:
    return json.dumps(
        {
            "bomFormat": "CycloneDX",
            "specVersion": "1.5",
            "metadata": {"component": {"name": "CopyPolish", "version": version}},
            "components": [{"name": f"dep-{i}", "version": "1.0.0"} for i in range(components)],
        }
    )


def write_asset(dist: Path, name: str, content: bytes) -> str:
    (dist / name).write_bytes(content)
    return hashlib.sha256(content).hexdigest()


class CheckSbomTest(unittest.TestCase):
    def test_valid_sbom_passes(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            sbom = Path(tmp) / vra.SBOM_NAME
            sbom.write_text(make_sbom(), encoding="utf-8")
            errors: list[str] = []
            vra.check_sbom(sbom, "v0.6.2", errors)
            self.assertEqual(errors, [])

    def test_invalid_json_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            sbom = Path(tmp) / vra.SBOM_NAME
            sbom.write_text("{not json", encoding="utf-8")
            errors: list[str] = []
            vra.check_sbom(sbom, "v0.6.2", errors)
            self.assertTrue(any("不是合法 JSON" in e for e in errors))

    def test_wrong_bom_format_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            sbom = Path(tmp) / vra.SBOM_NAME
            sbom.write_text(make_sbom().replace("CycloneDX", "SPDX"), encoding="utf-8")
            errors: list[str] = []
            vra.check_sbom(sbom, "v0.6.2", errors)
            self.assertTrue(any("bomFormat" in e for e in errors))

    def test_empty_components_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            sbom = Path(tmp) / vra.SBOM_NAME
            sbom.write_text(make_sbom(components=0), encoding="utf-8")
            errors: list[str] = []
            vra.check_sbom(sbom, "v0.6.2", errors)
            self.assertTrue(any("components" in e for e in errors))

    def test_version_mismatch_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            sbom = Path(tmp) / vra.SBOM_NAME
            sbom.write_text(make_sbom(version="0.6.1"), encoding="utf-8")
            errors: list[str] = []
            vra.check_sbom(sbom, "v0.6.2", errors)
            self.assertTrue(any("不一致" in e for e in errors))


class CheckSumsTest(unittest.TestCase):
    # SHA256SUMS 不包含自身条目。
    EXPECTED = vra.EXPECTED_ASSETS + (vra.SBOM_NAME,)

    def build_dist(self, dist: Path) -> dict[str, str]:
        digests = {}
        for name in vra.EXPECTED_ASSETS:
            digests[name] = write_asset(dist, name, f"asset:{name}".encode())
        digests[vra.SBOM_NAME] = write_asset(dist, vra.SBOM_NAME, make_sbom().encode())
        return digests

    def write_sums(self, dist: Path, digests: dict[str, str]) -> None:
        lines = [f"{digest}  {name}" for name, digest in sorted(digests.items())]
        (dist / vra.SUMS_NAME).write_text("\n".join(lines) + "\n", encoding="utf-8")

    def test_complete_sums_pass(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            dist = Path(tmp)
            self.write_sums(dist, self.build_dist(dist))
            errors: list[str] = []
            vra.check_sums(dist, self.EXPECTED, errors)
            self.assertEqual(errors, [])

    def test_missing_entry_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            dist = Path(tmp)
            digests = self.build_dist(dist)
            del digests[vra.SBOM_NAME]
            self.write_sums(dist, digests)
            errors: list[str] = []
            vra.check_sums(dist, self.EXPECTED, errors)
            self.assertTrue(any(f"缺少条目: {vra.SBOM_NAME}" in e for e in errors))

    def test_unexpected_entry_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            dist = Path(tmp)
            digests = self.build_dist(dist)
            digests["evil.txt"] = "0" * 64
            self.write_sums(dist, digests)
            errors: list[str] = []
            vra.check_sums(dist, self.EXPECTED, errors)
            self.assertTrue(any("非预期条目: evil.txt" in e for e in errors))

    def test_tampered_digest_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            dist = Path(tmp)
            digests = self.build_dist(dist)
            victim = vra.EXPECTED_ASSETS[0]
            (dist / victim).write_bytes(b"tampered")
            self.write_sums(dist, digests)
            errors: list[str] = []
            vra.check_sums(dist, self.EXPECTED, errors)
            self.assertTrue(any(f"SHA256 不匹配: {victim}" in e for e in errors))

    def test_duplicate_entry_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            dist = Path(tmp)
            digests = self.build_dist(dist)
            name, digest = next(iter(digests.items()))
            sums = dist / vra.SUMS_NAME
            self.write_sums(dist, digests)
            with sums.open("a", encoding="utf-8") as fh:
                fh.write(f"{digest}  {name}\n")
            errors: list[str] = []
            vra.check_sums(dist, self.EXPECTED, errors)
            self.assertTrue(any(f"重复条目: {name}" in e for e in errors))

    def test_path_traversal_name_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            dist = Path(tmp)
            digests = self.build_dist(dist)
            lines = [f"{'0' * 64}  ../../etc/passwd"]
            lines += [f"{d}  {n}" for n, d in sorted(digests.items())]
            (dist / vra.SUMS_NAME).write_text("\n".join(lines) + "\n", encoding="utf-8")
            errors: list[str] = []
            vra.check_sums(dist, self.EXPECTED, errors)
            self.assertTrue(any("不安全的资产名" in e for e in errors))

    def test_malformed_line_fails(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            dist = Path(tmp)
            self.build_dist(dist)
            (dist / vra.SUMS_NAME).write_text("not-a-checksum  CopyPolish.exe\n", encoding="utf-8")
            errors: list[str] = []
            vra.check_sums(dist, self.EXPECTED, errors)
            self.assertTrue(any("格式非法" in e for e in errors))


class CheckAssetsAllowlistTest(unittest.TestCase):
    def test_metadata_names_allowed_when_expected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            dist = Path(tmp)
            for name in vra.EXPECTED_ASSETS:
                (dist / name).write_bytes(b"x")
            (dist / vra.SBOM_NAME).write_text("{}", encoding="utf-8")
            (dist / vra.SUMS_NAME).write_text("", encoding="utf-8")
            errors: list[str] = []
            expected = vra.EXPECTED_ASSETS + vra.METADATA_NAMES
            vra.check_assets(dist, errors, "all", expected)
            self.assertEqual(errors, [])

    def test_metadata_names_rejected_without_expected(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            dist = Path(tmp)
            for name in vra.EXPECTED_ASSETS:
                (dist / name).write_bytes(b"x")
            (dist / vra.SBOM_NAME).write_text("{}", encoding="utf-8")
            errors: list[str] = []
            vra.check_assets(dist, errors, "all")
            self.assertTrue(any(vra.SBOM_NAME in e for e in errors))


if __name__ == "__main__":
    unittest.main()
