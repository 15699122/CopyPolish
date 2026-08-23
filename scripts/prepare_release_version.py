#!/usr/bin/env python3
"""把发布 tag 的完整版本号写入所有构建配置（供 release 工作流调用）。

稳定 tag（v0.4.0）写入 0.4.0；预发布 tag（v0.4.0-pre.3）写入完整
0.4.0-pre.3，保证应用内 `getVersion()` / 界面显示与 GitHub tag 一致。
仅在 CI runner 工作区执行，不回写源码提交。

用法：python3 scripts/prepare_release_version.py vX.Y.Z[-suffix]
"""

from __future__ import annotations

import io
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent


def main() -> int:
    if len(sys.argv) != 2:
        print("用法：python3 scripts/prepare_release_version.py vX.Y.Z[-suffix]")
        return 1
    tag = sys.argv[1]
    m = re.fullmatch(r"v(\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?)", tag)
    if not m:
        print(f"ERROR: 非法 tag 格式: {tag}")
        return 1
    version = m.group(1)

    # frontend/package.json
    pkg_path = ROOT / "frontend/package.json"
    pkg = json.loads(pkg_path.read_text(encoding="utf-8"))
    pkg["version"] = version
    pkg_path.write_text(json.dumps(pkg, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    # frontend/package-lock.json（根与 packages[""] 两处）
    lock_path = ROOT / "frontend/package-lock.json"
    lock = json.loads(lock_path.read_text(encoding="utf-8"))
    lock["version"] = version
    if "" in lock.get("packages", {}):
        lock["packages"][""]["version"] = version
    lock_path.write_text(json.dumps(lock, indent=2) + "\n", encoding="utf-8")

    # src-tauri/tauri.conf.json（Tauri getVersion() 的来源）
    conf_path = ROOT / "src-tauri/tauri.conf.json"
    conf = json.loads(conf_path.read_text(encoding="utf-8"))
    conf["version"] = version
    conf_path.write_text(json.dumps(conf, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")

    # src-tauri/Cargo.toml
    cargo_path = ROOT / "src-tauri/Cargo.toml"
    cargo = cargo_path.read_text(encoding="utf-8")
    cargo, n = re.subn(r'(?m)^version\s*=\s*"[^"]+"', f'version = "{version}"', cargo, count=1)
    if n != 1:
        print("ERROR: 未在 Cargo.toml 中找到包版本行")
        return 1
    cargo_path.write_text(cargo, encoding="utf-8")

    # src-tauri/Cargo.lock 中本包条目（存在时同步，避免构建时意外改动锁文件）
    cargo_lock_path = ROOT / "src-tauri/Cargo.lock"
    if cargo_lock_path.exists():
        text = cargo_lock_path.read_text(encoding="utf-8")
        pattern = re.compile(
            r'(\[\[package\]\]\nname = "chinese-copywriting-formatter"\nversion = ")[^"]+(")'
        )
        updated, n = pattern.subn(rf"\g<1>{version}\g<2>", text)
        if n:
            cargo_lock_path.write_text(updated, encoding="utf-8")

    # Windows runner 的 stdout 可能是 cp1252 等非 UTF-8 编码，输出保持纯 ASCII。
    if isinstance(sys.stdout, io.TextIOWrapper):
        sys.stdout.reconfigure(encoding="utf-8")
    print(f"OK: synced release version {version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
