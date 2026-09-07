"""SBOM 生成器的元数据回归测试。"""

from __future__ import annotations

import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


class GenerateSbomTest(unittest.TestCase):
    def test_output_contains_application_component_version(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            output = Path(tmp) / "sbom.json"
            result = subprocess.run(
                [sys.executable, "scripts/generate_sbom.py", "--output", str(output)],
                cwd=ROOT,
                check=True,
                capture_output=True,
                text=True,
            )
            self.assertIn("SBOM written", result.stdout)
            document = json.loads(output.read_text(encoding="utf-8"))
            component = document["metadata"]["component"]
            self.assertEqual(component["name"], "chinese-copywriting-formatter")
            self.assertEqual(component["version"], "0.6.2-dev.1")

    def test_check_mode_still_validates_generated_document(self) -> None:
        result = subprocess.run(
            [sys.executable, "scripts/generate_sbom.py", "--check"],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        )
        self.assertIn("SBOM check OK:", result.stdout)


if __name__ == "__main__":
    unittest.main()