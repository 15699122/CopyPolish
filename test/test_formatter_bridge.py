# -*- coding: utf-8 -*-
"""formatter_bridge 单元测试（不依赖 GUI）。

运行：.venv/bin/python -m unittest -v test.test_formatter_bridge
"""
import os
import sys
import tempfile
import unittest

_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
_PY = os.path.join(_ROOT, "python")
for _p in (_ROOT, _PY):
    if _p not in sys.path:
        sys.path.insert(0, _p)

import ccw_engine
from formatter_bridge import enabled_defaults, format_document, list_rules


class TestFormatDocument(unittest.TestCase):
    def test_basic_format(self):
        self.assertEqual(
            format_document("在LeanCloud上，花了5000元"),
            "在 LeanCloud 上，花了 5000 元",
        )

    def test_none_enables_all(self):
        self.assertEqual(format_document("github！！", None), "GitHub！")

    def test_baseline_spacing_persists_with_empty_enabled(self):
        # 固有收尾规则（CJK-拉丁空格）不随规则开关关闭
        self.assertEqual(format_document("在LeanCloud上", []), "在 LeanCloud 上")

    def test_toggleable_rules_respect_empty_enabled(self):
        # 可开关规则（重复标点、专有名词）在空启用集下不生效
        self.assertEqual(format_document("巴西队！！", []), "巴西队！！")
        self.assertEqual(format_document("github", []), "github")

    def test_invalid_keys_do_not_apply_toggleable_rules(self):
        self.assertEqual(format_document("github！！", ["不存在的规则"]), "github！！")


class TestRuleMetadata(unittest.TestCase):
    def test_list_rules_shape(self):
        rules = list_rules()
        self.assertEqual(len(rules), 13)
        for rule in rules:
            for field in ("key", "section", "name", "disputed", "default"):
                self.assertIn(field, rule, rule)

    def test_defaults_exclude_disputed(self):
        defaults = set(enabled_defaults())
        rules = {r["key"]: r for r in list_rules()}
        for key, rule in rules.items():
            expected = rule["default"]
            self.assertEqual(key in defaults, expected, (key, expected))

    def test_defaults_are_sorted(self):
        self.assertEqual(enabled_defaults(), sorted(enabled_defaults()))


class TestExplicitInitialize(unittest.TestCase):
    def test_initialize_creates_defaults_in_given_path(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, "sub", "rules.yaml")
            ccw_engine.initialize(path)
            data = ccw_engine.load_yaml(path)
            self.assertIn("settings", data)
            self.assertIn("rules", data)


if __name__ == "__main__":
    unittest.main()
