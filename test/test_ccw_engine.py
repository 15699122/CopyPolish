# -*- coding: utf-8 -*-
"""ccw_engine 单元测试（不依赖 GUI）。运行：python3 -m unittest -v test.test_ccw_engine"""
import os
import sys

# 让本文件无论从项目根还是直接执行都能 import 到父目录的 ccw_engine（/ccw_engine.py）。
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import unittest

from ccw_engine import (
    RULES,
    dump_yaml,
    format_text,
    get_enabled_defaults,
    get_rule_by_key,
    load_rules,
    load_settings,
    save_settings,
)

ENABLED = get_enabled_defaults()


def _key_of(name):
    """获取规则键值；若未找到则断言失败（避免 Pylance Optional 下标告警）。"""
    rule = get_rule_by_key(name)
    assert rule is not None, "未找到规则: %s" % name
    return rule["key"]


class TestSpaceRules(unittest.TestCase):
    def test_cn_en_space(self):
        self.assertEqual(format_text("在LeanCloud上", ENABLED), "在 LeanCloud 上")
        self.assertEqual(format_text("数据是schema-free的", ENABLED), "数据是 schema-free 的")
        self.assertEqual(format_text("在 LeanCloud 上", ENABLED), "在 LeanCloud 上")

    def test_cn_digit_space(self):
        self.assertEqual(format_text("花了5000元", ENABLED), "花了 5000 元")
        self.assertEqual(format_text("买了 5000元", ENABLED), "买了 5000 元")

    def test_digit_unit_space(self):
        self.assertEqual(format_text("宽带有 10Gbps", ENABLED), "宽带有 10 Gbps")
        self.assertEqual(format_text("SSD 一共有 20TB", ENABLED), "SSD 一共有 20 TB")
        self.assertEqual(format_text("角度为 90 ° 的角", ENABLED), "角度为 90° 的角")
        self.assertEqual(format_text("有 15 % 的 CPU", ENABLED), "有 15% 的 CPU")

    def test_fw_punct_no_space(self):
        self.assertEqual(format_text("买了一部 iPhone ，好开心！", ENABLED), "买了一部 iPhone，好开心！")
        self.assertEqual(format_text("买了一部 iPhone， 好开心！", ENABLED), "买了一部 iPhone，好开心！")


class TestPunctuationRules(unittest.TestCase):
    def test_no_repeat_punct(self):
        self.assertEqual(format_text("德国队竟然战胜了巴西队！！", ENABLED), "德国队竟然战胜了巴西队！")
        self.assertEqual(format_text("巴西队！！！！！！！！", ENABLED), "巴西队！")
        self.assertEqual(format_text("她竟然对你说「喵」？？！！", ENABLED), "她竟然对你说「喵」？！")
        self.assertEqual(format_text("她竟然对你说「喵」？！？！？？！！", ENABLED), "她竟然对你说「喵」？！")

    def test_fullwidth_chinese_punct(self):
        self.assertEqual(
            format_text('嗨! 你知道嘛? 今天前台的小妹跟我说 "喵" 了哎!', ENABLED),
            "嗨！你知道嘛？今天前台的小妹跟我说「喵」了哎！")
        self.assertEqual(
            format_text("核磁共振成像 (NMRI) 是什么原理都不知道? JFGI!", ENABLED),
            "核磁共振成像（NMRI）是什么原理都不知道？JFGI！")
        self.assertEqual(format_text("数字 5:30 和 1,000 不变", ENABLED), "数字 5:30 和 1,000 不变")

    def test_fullwidth_digits(self):
        self.assertEqual(format_text("只卖 １０００ 元", ENABLED), "只卖 1000 元")

    def test_halfwidth_in_english(self):
        self.assertEqual(format_text("「Stay hungry，stay foolish。」", ENABLED),
                         "「Stay hungry, stay foolish.」")
        self.assertEqual(
            format_text("《Hackers＆Painters：Big Ideas from the Computer Age》", ENABLED),
            "《Hackers & Painters: Big Ideas from the Computer Age》")


class TestNounRules(unittest.TestCase):
    def test_proper_nouns(self):
        for wrong in ("github", "GITHUB", "Github", "gitHub"):
            self.assertEqual(format_text("使用 %s 登录" % wrong, ENABLED), "使用 GitHub 登录")
        self.assertEqual(format_text("我们的客户有 facebook, inc.。", ENABLED),
                         "我们的客户有 Facebook, Inc.。")

    def test_no_abbr(self):
        self.assertEqual(
            format_text("熟悉 Ts、h5，以及 RJS、nextjs 的 FED", ENABLED),
            "熟悉 TypeScript、HTML5，以及 React、Next.js 的前端开发者")


class TestDisputedRules(unittest.TestCase):
    def test_off_by_default(self):
        self.assertNotIn(_key_of("链接之间增加空格"), ENABLED)
        self.assertNotIn(_key_of("简体中文使用直角引号"), ENABLED)

    def test_space_around_links(self):
        on = ENABLED | {_key_of("链接之间增加空格")}
        self.assertEqual(format_text("请[提交一个 issue](#)并分配", on),
                         "请 [提交一个 issue](#) 并分配")

    def test_corner_quotes(self):
        on = ENABLED | {_key_of("简体中文使用直角引号")}
        self.assertEqual(format_text("“老师，‘有条不紊’的‘紊’是什么意思？”", on),
                         "「老师，『有条不紊』的『紊』是什么意思？」")


class TestProtection(unittest.TestCase):
    def test_url_protected(self):
        src = "访问 https://example.com/a?b=1，下载。"
        self.assertEqual(format_text(src, ENABLED), "访问 https://example.com/a?b=1，下载。")

    def test_email_protected(self):
        self.assertIn("me@example.com", format_text("联系me@example.com获取", ENABLED))

    def test_inline_code_protected(self):
        self.assertEqual(format_text("使用`npm install -g x`安装", ENABLED),
                         "使用 `npm install -g x` 安装")


class TestStability(unittest.TestCase):
    def test_idempotent(self):
        src = "在LeanCloud上，数据存储是围绕`AVObject`进行的。花了5000元。"
        once = format_text(src, ENABLED)
        self.assertEqual(format_text(once, ENABLED), once)

    def test_rule_count(self):
        self.assertEqual(len(RULES), 13)

    def test_sections(self):
        sections = [r["section"] for r in RULES]
        self.assertEqual(sections.count("空格"), 5)
        self.assertEqual(sections.count("标点符号"), 1)
        self.assertEqual(sections.count("全角和半角"), 3)
        self.assertEqual(sections.count("名词"), 2)
        self.assertEqual(sections.count("争议"), 2)


class TestYamlRules(unittest.TestCase):
    """rules.yaml 规则装载与设置持久化。"""

    def test_load_rules_count(self):
        rules = load_rules()
        self.assertEqual(len(rules), 13)

    def test_load_rules_has_default(self):
        rules = load_rules()
        self.assertTrue(all("default" in r for r in rules))
        by_key = {r["key"]: r for r in rules}
        # 争议规则 default=False
        self.assertFalse(by_key["链接之间增加空格"]["default"])
        self.assertFalse(by_key["简体中文使用直角引号"]["default"])
        # 普通规则 default=True
        self.assertTrue(by_key["中英文之间需要增加空格"]["default"])

    def test_missing_yaml_falls_back(self):
        rules = load_rules("/nonexistent/rules.yaml")
        self.assertEqual(len(rules), 13)

    def test_settings_roundtrip(self):
        import tempfile
        import os
        fd, path = tempfile.mkstemp(suffix=".yaml")
        os.close(fd)
        try:
            self.assertTrue(save_settings(
                {"不重复使用标点符号", "使用全角中文标点"}, "你好，world", path))
            enabled, last = load_settings(path)
            self.assertEqual(enabled, {"不重复使用标点符号", "使用全角中文标点"})
            self.assertEqual(last, "你好，world")
        finally:
            os.remove(path)

    def test_settings_roundtrip_dump_yaml_helper(self):
        import tempfile
        import os
        fd, path = tempfile.mkstemp(suffix=".yaml")
        os.close(fd)
        try:
            self.assertTrue(dump_yaml(path, {"a": 1, "b": "x"}))
            self.assertTrue(os.path.getsize(path) > 0)
        finally:
            os.remove(path)


class TestLineStructure(unittest.TestCase):
    def test_empty_lines_are_preserved(self):
        self.assertEqual(format_text("第一段\n\n第二段", ENABLED), "第一段\n\n第二段")
        self.assertEqual(format_text("\n第一段\n\n\n第二段\n", ENABLED), "\n第一段\n\n\n第二段\n")

    def test_blank_lines_are_normalized_but_kept(self):
        self.assertEqual(format_text("第一段\n   \n第二段", ENABLED), "第一段\n\n第二段")

    def test_newline_style_is_preserved(self):
        self.assertEqual(format_text("在LeanCloud上\r\n\r\n花了5000元", ENABLED),
                         "在 LeanCloud 上\r\n\r\n花了 5000 元")
        self.assertEqual(format_text("在LeanCloud上\r花了5000元", ENABLED),
                         "在 LeanCloud 上\r花了 5000 元")


class TestLatexProtection(unittest.TestCase):
    def test_inline_math_dollar(self):
        self.assertEqual(format_text("公式$E=mc^2$很重要", ENABLED), "公式 $E=mc^2$ 很重要")

    def test_inline_math_paren(self):
        self.assertEqual(format_text(r"公式\( E=mc^2 \)很重要", ENABLED), r"公式 \( E=mc^2 \) 很重要")

    def test_display_math_is_unchanged(self):
        src = "如下：\n$$\nE=mc^2; github\n$$\n结束"
        self.assertEqual(format_text(src, ENABLED), src)

    def test_latex_environment_is_unchanged(self):
        src = "如下：\n\\begin{align}\na&=b+c; github\n\\end{align}\n结束"
        self.assertEqual(format_text(src, ENABLED), src)

    def test_latex_command_is_unchanged(self):
        src = r"使用\frac{a}{b}计算"
        self.assertEqual(format_text(src, ENABLED), r"使用 \frac{a}{b} 计算")

    def test_escaped_dollar_is_not_math(self):
        self.assertEqual(format_text(r"价格是\$100", ENABLED), r"价格是\$100")


class TestMarkdownProtection(unittest.TestCase):
    def test_fenced_code_block_is_unchanged(self):
        src = "示例：\n```python\nprint('github; $x | y')\n```\n结束"
        self.assertEqual(format_text(src, ENABLED), src)

    def test_inline_code_is_unchanged_inside(self):
        self.assertEqual(format_text("使用`a;b|c/$x`安装", ENABLED), "使用 `a;b|c/$x` 安装")

    def test_markdown_link_is_protected(self):
        src = "请看[GitHub链接](https://example.com/a;b?x=$1|y)然后继续"
        self.assertEqual(format_text(src, ENABLED), "请看 [GitHub链接](https://example.com/a;b?x=$1|y) 然后继续")

    def test_markdown_image_is_protected(self):
        src = "图片![alt text](image/path.png \"title\")很好"
        self.assertEqual(format_text(src, ENABLED), "图片 ![alt text](image/path.png \"title\") 很好")

    def test_indented_code_line_is_unchanged(self):
        src = "命令：\n    npm install foo/bar; echo '$x|y'\n完成"
        self.assertEqual(format_text(src, ENABLED), src)


class TestSpecialCharacters(unittest.TestCase):
    def test_special_characters_are_not_lost(self):
        cases = [
            "价格是 $100",
            "执行 npm install foo/bar",
            r"路径是 C:\Users\Test",
            "条件为 a|b",
            "it's useful",
            "A;B;C",
            "使用 `a;b|c`",
        ]
        for src in cases:
            out = format_text(src, ENABLED)
            for ch in "$`;'/\\|":
                if ch in src:
                    self.assertIn(ch, out, (src, out, ch))

    def test_new_cases_are_idempotent(self):
        cases = [
            "第一段\n\n第二段",
            "公式$E=mc^2$很重要",
            "示例：\n```\ngithub; $x | y\n```\n结束",
            r"路径是 C:\Users\Test，价格是\$100",
        ]
        for src in cases:
            once = format_text(src, ENABLED)
            self.assertEqual(format_text(once, ENABLED), once, src)

if __name__ == "__main__":
    unittest.main()
