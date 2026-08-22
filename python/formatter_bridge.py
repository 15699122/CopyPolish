# -*- coding: utf-8 -*-
"""中文文案排版助手 · Rust/PyO3 桥接层。

为 Tauri + PyO3 内嵌 CPython 场景提供稳定、受限的调用入口：
只暴露格式化和规则元数据相关的纯函数，不在导入阶段做任何文件读写，
从而避免只读资源目录或嵌入解释器下的隐式副作用。

调用契约（供 Rust commands 对应）：
    format_document(text: str, enabled: list[str] | None) -> str
    list_rules(config_path: str | None = None) -> list[dict]
    enabled_defaults() -> list[str]

配置读写（settings.json / rules.yaml 的用户状态部分）由 Rust 侧负责，
本模块不承担持久化职责。
"""

from __future__ import annotations

import os
import sys

# 让本模块无论是在项目根直接运行、还是作为 Tauri 资源被 PYTHONPATH 导入时，
# 都能解析到项目根目录下的 ccw_engine.py。
_HERE = os.path.dirname(os.path.abspath(__file__))
_ROOT = os.path.dirname(_HERE)
if _ROOT not in sys.path:
    sys.path.insert(0, _ROOT)

from ccw_engine import (  # noqa: E402
    format_text,
    get_enabled_defaults,
    load_rules,
)


def format_document(text: str, enabled: list[str] | None = None) -> str:
    """按给定的启用规则 key 集合规范化文本。

    enabled 为 None 表示全部规则启用；空列表表示不启用任何规则。
    input 中无效的 key 会被规则引擎忽略。
    """
    enabled_set: set[str] | None = set(enabled) if enabled is not None else None
    return format_text(text, enabled_set)


def list_rules(config_path: str | None = None) -> list[dict]:
    """返回规则元数据（不含 Python 函数实现），便于前端渲染设置页。"""
    rules = load_rules(config_path)
    return [
        {
            "key": r["key"],
            "section": r["section"],
            "name": r["name"],
            "disputed": bool(r["disputed"]),
            "default": bool(r["default"]),
        }
        for r in rules
    ]


def enabled_defaults() -> list[str]:
    """返回默认启用的规则 key 列表（排序稳定，利于 diff/缓存）。"""
    return sorted(get_enabled_defaults())


if __name__ == "__main__":  # 简单自检，便于无 GUI 环境验证桥接层
    import json

    sample = "在LeanCloud上，花了5000元"
    print("format_document:", json.dumps(format_document(sample), ensure_ascii=False))
    print("enabled_defaults:", enabled_defaults())
    print("rules_count:", len(list_rules()))
