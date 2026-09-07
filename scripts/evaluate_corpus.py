#!/usr/bin/env python3
"""本地来源文本清洗语料评测脚本。

读取 src-tauri/tests/fixtures/corpus-local/ 中的 YAML 语料，
对指定规则执行格式化，并与期望输出比较，统计 TP/FP/FN。
真实语料不提交仓库；本脚本仅在本地运行。

用法：
  python3 scripts/evaluate_corpus.py \
    --fixtures src-tauri/tests/fixtures/corpus-local/ \
    --rules cleanup.cjk-internal-space
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


SAMPLE_FIELDS = ("id", "source_type", "input", "expected", "annotation")


@dataclass
class Sample:
    id: str
    source_type: str
    input: str
    expected: str
    annotation: str
    raw: dict


def load_samples(fixtures_dir: Path) -> list[Sample]:
    samples: list[Sample] = []
    for path in sorted(fixtures_dir.glob("*.yaml")):
        import yaml

        with path.open(encoding="utf-8") as fh:
            data = yaml.safe_load(fh) or []
        for raw in data:
            if not isinstance(raw, dict):
                continue
            samples.append(
                Sample(
                    id=str(raw.get("id", path.stem)),
                    source_type=str(raw.get("source_type", "unknown")),
                    input=str(raw.get("input", "")),
                    expected=str(raw.get("expected", "")),
                    annotation=str(raw.get("annotation", "")),
                    raw=raw,
                )
            )
    return samples


def run_format(rust_bin: Path, text: str, rules: list[str]) -> str:
    """通过 copypolish-tui 非交互模式执行格式化。"""
    args = [
        str(rust_bin),
        "--stdin",
        "--no-config",
        "--rules",
        "none",
    ]
    for rule in rules:
        args.extend(["--enable", rule])

    proc = subprocess.run(
        args,
        input=text,
        capture_output=True,
        text=True,
        encoding="utf-8",
        check=False,
    )
    if proc.returncode != 0:
        raise RuntimeError(
            f"copypolish-tui 退出码 {proc.returncode}: {proc.stderr.strip()}"
        )
    return proc.stdout.rstrip("\n")


def find_diff_regions(actual: str, expected: str) -> list[tuple[int, int]]:
    """返回 actual 中与 expected 不同的字符区间（简化版）。"""
    regions: list[tuple[int, int]] = []
    min_len = min(len(actual), len(expected))
    start = None
    for i in range(min_len):
        if actual[i] != expected[i]:
            if start is None:
                start = i
        elif start is not None:
            regions.append((start, i))
            start = None
    if start is not None:
        regions.append((start, min_len))
    if len(actual) > len(expected):
        regions.append((len(expected), len(actual)))
    elif len(expected) > len(actual):
        regions.append((len(actual), len(expected)))
    return regions


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--fixtures",
        type=Path,
        default=Path("src-tauri/tests/fixtures/corpus-local"),
        help="语料目录路径",
    )
    parser.add_argument(
        "--rules",
        nargs="+",
        required=True,
        help="要评测的规则 key 列表",
    )
    parser.add_argument(
        "--bin",
        type=Path,
        help="copypolish-tui 二进制路径（默认自动查找）",
    )
    args = parser.parse_args()

    samples = load_samples(args.fixtures)
    if not samples:
        print(f"未在 {args.fixtures} 找到语料", file=sys.stderr)
        return 1

    rust_bin = args.bin
    if rust_bin is None:
        candidate = (
            Path("src-tauri/target/debug/copypolish-tui")
        )
        if not candidate.exists():
            print(
                "未找到 copypolish-tui，请先构建："
                "cargo build --manifest-path src-tauri/Cargo.toml "
                "--features tui --bin copypolish-tui",
                file=sys.stderr,
            )
            return 1
        rust_bin = candidate

    tp = fp = fn = 0
    results = []

    for sample in samples:
        try:
            actual = run_format(rust_bin, sample.input, args.rules)
        except RuntimeError as exc:
            results.append(
                {
                    "id": sample.id,
                    "status": "error",
                    "error": str(exc),
                }
            )
            continue

        expected = sample.expected.rstrip("\n")
        if actual == expected:
            tp += 1
            status = "tp"
        else:
            # 简化判定：若输出与期望不同，记为 fp 或 fn
            # 真实场景需人工复核
            regions = find_diff_regions(actual, expected)
            if len(actual) < len(expected):
                fn += 1
                status = "fn"
            else:
                fp += 1
                status = "fp"

        results.append(
            {
                "id": sample.id,
                "source_type": sample.source_type,
                "status": status,
                "input": sample.input,
                "expected": expected,
                "actual": actual,
                "diff_regions": regions if status != "tp" else [],
                "annotation": sample.annotation,
            }
        )

    total = len(samples)
    summary = {
        "total": total,
        "tp": tp,
        "fp": fp,
        "fn": fn,
        "accuracy": tp / total if total else 0.0,
        "fp_rate": fp / total if total else 0.0,
    }

    print(json.dumps({"summary": summary, "results": results}, ensure_ascii=False, indent=2))

    if fp > 0:
        return 2  # 存在误改，需要人工复核
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
