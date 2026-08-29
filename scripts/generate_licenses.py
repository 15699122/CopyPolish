#!/usr/bin/env python3
"""从锁定依赖元数据生成第三方许可证清单。"""

from __future__ import annotations

import json
import subprocess
import sys
from collections import defaultdict
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
OUTPUT = ROOT / "docs" / "licenses.md"
MANIFEST = ROOT / "src-tauri" / "Cargo.toml"
FRONTEND_MODULES = ROOT / "frontend" / "node_modules"


def cargo_packages() -> list[tuple[str, str, str]]:
    result = subprocess.run(
        ["cargo", "metadata", "--manifest-path", str(MANIFEST), "--locked", "--format-version", "1"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    )
    metadata = json.loads(result.stdout)
    rows = []
    for package in metadata["packages"]:
        if package["name"] == "chinese-copywriting-formatter" and package["source"] is None:
            continue
        rows.append((package["name"], package["version"], package.get("license") or "UNKNOWN"))
    return sorted(set(rows), key=lambda row: (row[0], row[1], row[2]))


def npm_packages() -> list[tuple[str, str, str]]:
    if not FRONTEND_MODULES.is_dir():
        raise RuntimeError("frontend/node_modules 不存在，请先运行 npm ci --prefix frontend")

    rows: dict[tuple[str, str], str] = {}
    package_files = list(FRONTEND_MODULES.glob("*/package.json"))
    package_files.extend(FRONTEND_MODULES.glob("@*/*/package.json"))
    for path in package_files:
        package = json.loads(path.read_text(encoding="utf-8"))
        name = package.get("name", path.parent.name)
        version = package.get("version", "UNKNOWN")
        license_name = package.get("license")
        if not license_name and package.get("licenses"):
            license_name = "SEE LICENSE"
        rows[(name, version)] = license_name or "UNKNOWN"
    return sorted(
        [(name, version, license_name) for (name, version), license_name in rows.items()],
        key=lambda row: (row[0], row[1]),
    )


def table(rows: list[tuple[str, str, str]]) -> str:
    lines = ["| 包 | 版本 | 许可证 |", "|---|---:|---|"]
    lines.extend(f"| `{name}` | `{version}` | `{license_name}` |" for name, version, license_name in rows)
    return "\n".join(lines)


def generate() -> None:
    rust = cargo_packages()
    npm = npm_packages()
    all_rows = rust + npm
    unknown = [(source, row) for source, rows in (("Rust", rust), ("npm", npm)) for row in rows if row[2] == "UNKNOWN"]
    license_counts: dict[str, int] = defaultdict(int)
    for _, _, license_name in all_rows:
        license_counts[license_name] += 1

    lines = [
        "# 第三方许可证清单",
        "",
        "本清单由 `python3 scripts/generate_licenses.py` 生成，不手工编辑。Rust 依赖来自",
        "`src-tauri/Cargo.lock` 对应的 `cargo metadata --locked`；npm 依赖来自",
        "`frontend/package-lock.json` 安装后的 `frontend/node_modules/**/package.json`。",
        "",
        "> 生成日期：由脚本运行时写入；依赖升级后必须重新生成并审阅差异。",
        "",
        "## 汇总",
        "",
        f"- Rust 依赖：{len(rust)} 条（含不同版本的同名包）；",
        f"- npm 依赖：{len(npm)} 条；",
        f"- 许可证字段缺失：{len(unknown)} 条。",
        "",
        "| 许可证字段 | 数量 |",
        "|---|---:|",
    ]
    lines.extend(f"| `{license_name}` | {count} |" for license_name, count in sorted(license_counts.items()))
    lines.extend(["", "## Rust 依赖", "", table(rust), "", "## npm 依赖", "", table(npm)])
    if unknown:
        lines.extend(["", "## 缺失许可证字段", "", "以下条目需要在后续依赖升级或发布审阅中人工确认：", ""])
        lines.extend(f"- {source}: `{name}` `{version}`" for source, (name, version, _) in unknown)
    OUTPUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"generated {OUTPUT.relative_to(ROOT)} ({len(rust)} Rust, {len(npm)} npm)")


if __name__ == "__main__":
    try:
        generate()
    except (OSError, RuntimeError, subprocess.CalledProcessError, json.JSONDecodeError) as error:
        print(f"许可证清单生成失败：{error}", file=sys.stderr)
        raise SystemExit(1)