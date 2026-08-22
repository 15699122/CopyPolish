# -*- coding: utf-8 -*-
"""Python vs Rust 双引擎 parity 对比。

运行：.venv/bin/python test/compare_rust_parity.py
对同一语料分别用 ccw_engine.py 与 src-tauri Rust 引擎格式化，
逐字对比输出；不一致时打印双方结果。
"""
import json
import os
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.insert(0, ROOT)

import ccw_engine  # noqa: E402

DEFAULTS = sorted(ccw_engine.get_enabled_defaults())

CASES = [
    # 基础空格 / 单位
    "在LeanCloud上",
    "在 LeanCloud 上",
    "数据是schema-free的",
    "花了5000元",
    "买了 5000元",
    "宽带有 10Gbps",
    "SSD 一共有 20TB",
    "角度为 90 ° 的角",
    "有 15 % 的 CPU",
    "买了一部 iPhone ，好开心！",
    "买了一部 iPhone， 好开心！",
    # 标点
    "德国队竟然战胜了巴西队！！",
    "巴西队！！！！！！！！",
    "她竟然对你说「喵」？？！！",
    '嗨! 你知道嘛? 今天前台的小妹跟我说 "喵" 了哎!',
    "核磁共振成像 (NMRI) 是什么原理都不知道? JFGI!",
    "数字 5:30 和 1,000 不变",
    "只卖 １０００ 元",
    "「Stay hungry，stay foolish。」",
    "《Hackers＆Painters：Big Ideas from the Computer Age》",
    # 名词 / 缩写
    "使用 github 登录",
    "使用 GITHUB 登录",
    "我们的客户有 facebook, inc.。",
    "熟悉 Ts、h5，以及 RJS、ne",
    # 换行
    "第一段\n\n第二段",
    "\n第一段\n\n\n第二段\n",
    "第一段\n   \n第二段",
    "在LeanCloud上\r\n\r\n花了5000元",
    "在LeanCloud上\r花了5000元",
    # LaTeX 保护
    "公式$E=mc^2$很重要",
    r"公式\( E=mc^2 \)很重要",
    "如下：\n$$\nE=mc^2; github\n$$\n结束",
    "如下：\n\\begin{align}\na&=b+c; github\n\\end{align}\n结束",
    r"使用\frac{a}{b}计算",
    r"价格是\$100",
    # Markdown 保护
    "示例：\n```python\nprint('github; $x | y')\n```\n结束",
    "使用`a;b|c/$x`安装",
    "请看[GitHub链接](https://example.com/a;b?x=$1|y)然后继续",
    '图片![alt text](image/path.png "title")很好',
    "命令：\n    npm install foo/bar; echo '$x|y'\n完成",
    # 特殊字符与幂等
    "价格是 $100",
    "执行 npm install foo/bar",
    r"路径是 <windows-user-home>，价格是\$100",
    "条件为 a|b",
    "it's useful",
    "A;B;C",
    # 争议规则（all 模式下生效）
    "访问https://example.com/a?b=1|x获取详情",
    "联系 admin@example.com 或 visit https://example.com 了解",
    "他说\"你好世界\"然后离开",
    "他说“你好世界”然后离开",
]


def main() -> int:
    modes = [
        ("defaults", DEFAULTS),
        ("all", None),  # None -> format_text(text, None)，全部启用
    ]
    failures = 0
    for mode, enabled in modes:
        py_out = [ccw_engine.format_text(c, enabled) for c in CASES]
        payload = json.dumps(
            [{"text": c, "enabled": ([] if enabled is None else list(enabled))} for c in CASES],
            ensure_ascii=False,
        )
        proc = subprocess.run(
            ["cargo", "run", "--quiet", "--manifest-path",
             os.path.join("src-tauri", "Cargo.toml"), "--example", "parity_dump"],
            input=payload, capture_output=True, text=True,
            env={**os.environ, "PYO3_PYTHON": "/usr/bin/python3.14"},
        )
        if proc.returncode != 0:
            print(proc.stderr[-2000:])
            return 1
        rs_out = json.loads(proc.stdout.strip().splitlines()[-1])
        for case, p, r in zip(CASES, py_out, rs_out):
            if p != r:
                failures += 1
                print(f"[{mode}] MISMATCH input={case!r}")
                print(f"  python: {p!r}")
                print(f"  rust  : {r!r}")
        print(f"[{mode}] checked {len(CASES)} cases")
    print(f"total mismatches: {failures}")
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
