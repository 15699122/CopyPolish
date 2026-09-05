#!/usr/bin/env python3
"""运行 release 文本基准并拦截数量级性能回退。

该门禁不要求每次运行得到完全相同的毫秒数，只检查 1 MB 语料是否超过
明显高于当前基线的宽松上限。详细耗时、峰值 RSS 和历史对比仍记录在
``docs/benchmarks/unicode-baseline.md``。
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "src-tauri" / "Cargo.toml"
COMMAND = [
    "cargo",
    "run",
    "--release",
    "--manifest-path",
    str(MANIFEST),
    "--example",
    "unicode_baseline",
]

# 当前基线约为：普通语料 0.12–0.18 s，Markdown/LaTeX 约 1.6 s。
# 阈值留出构建机和运行时抖动空间，只拦截数量级回退。
LIMITS_MS = {
    "纯中文": 500.0,
    "中英数混排": 500.0,
    "Markdown/LaTeX 密集": 5000.0,
    "emoji/组合字符密集": 500.0,
    "CJK Ext-B 密集": 500.0,
}


def main() -> int:
    result = subprocess.run(
        COMMAND,
        cwd=ROOT,
        text=True,
        encoding="utf-8",
        errors="replace",
        capture_output=True,
        check=False,
    )
    output = result.stdout + result.stderr
    if result.returncode != 0:
        print(output, end="")
        print("performance gate: benchmark command failed", file=sys.stderr)
        return result.returncode or 1

    failures: list[str] = []
    observed: dict[str, float] = {}
    in_1mb = False
    pattern = re.compile(r"^\s*(?P<label>.+?)\s*@1024KB:\s+(?P<value>[0-9.]+)\s+ms/round$")
    for line in output.splitlines():
        if line.strip() == "--- 1024 KB ---":
            in_1mb = True
            continue
        if not in_1mb:
            continue
        match = pattern.match(line)
        if not match:
            continue
        label = match.group("label").strip()
        value = float(match.group("value"))
        observed[label] = value

    missing = sorted(set(LIMITS_MS) - set(observed))
    if missing:
        failures.append(f"missing 1 MB benchmark rows: {', '.join(missing)}")

    for label, limit in LIMITS_MS.items():
        value = observed.get(label)
        if value is not None and value > limit:
            failures.append(f"{label}: {value:.2f} ms exceeds {limit:.0f} ms limit")

    if failures:
        print("performance gate: FAIL", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        print("\nBenchmark output:", file=sys.stderr)
        print(output, file=sys.stderr, end="")
        return 1

    print("performance gate: PASS")
    for label in LIMITS_MS:
        print(f"  - {label}: {observed[label]:.2f} ms/round (limit {LIMITS_MS[label]:.0f} ms)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())