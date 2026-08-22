# -*- coding: utf-8 -*-
"""tauri-plugin-python 的 Python 后端入口（PyO3 模式）。

约定：此处列出的函数可为前端调用；结合 Rust 侧注册做最小准入。
参考实现（以本机安装的插件版本 README 为准）：
    _tauri_plugin_functions = ["format_document"]
"""

import os
import sys

_tauri_plugin_functions = ["format_document", "list_rules", "enabled_defaults"]

# 让脚本无论位于开发树 `src-tauri/src-python`，还是 Tauri 打包资源目录
# `<resource>/src-python`，都能找到随包分发的 ccw_engine.py 与 rules.yaml。
_HERE = os.path.dirname(os.path.abspath(__file__))
_RESOURCE_ROOT = os.path.dirname(_HERE) if os.path.basename(_HERE) == "src-python" else _HERE
_UP_ROOT = os.path.join(_RESOURCE_ROOT, "_up_")
_PROJECT_ROOT = os.path.dirname(_RESOURCE_ROOT)

for _p in (_RESOURCE_ROOT, _UP_ROOT, _PROJECT_ROOT):
    if _p not in sys.path:
        sys.path.insert(0, _p)


def _first_existing_path(*paths):
    for path in paths:
        if path and os.path.exists(path):
            return path
    return paths[0] if paths else None


_RULES_PATH = _first_existing_path(
    os.path.join(_RESOURCE_ROOT, "rules.yaml"),
    os.path.join(_RESOURCE_ROOT, "_up_", "rules.yaml"),
    os.path.join(_PROJECT_ROOT, "rules.yaml"),
)


def _engine():
    import ccw_engine

    ccw_engine.initialize(_RULES_PATH)
    return ccw_engine


def format_document(text, enabled=None):
    engine = _engine()

    enabled_set = set(enabled) if enabled else None
    return engine.format_text(text, enabled_set)


def list_rules():
    engine = _engine()

    return [
        {
            "key": r["key"],
            "section": r["section"],
            "name": r["name"],
            "disputed": bool(r["disputed"]),
            "default": bool(r["default"]),
        }
        for r in engine.load_rules(_RULES_PATH)
    ]


def enabled_defaults():
    engine = _engine()

    return sorted(engine.get_enabled_defaults())
