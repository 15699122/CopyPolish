#!/usr/bin/env python3
"""生成 CycloneDX 1.5 SBOM（软件物料清单）。

依赖来源：
- Rust 生产依赖：`src-tauri/Cargo.toml` + `Cargo.lock`（经 `cargo metadata --locked`）；
  标记为 production（scope=required）。包本体（chinese-copywriting-formatter）跳过。
- 前端依赖：`frontend/package-lock.json` 的 `packages` 条目：
    - 生产依赖 scope=required；
    - devDependencies scope=optional，并在 `properties` 标记 `development=true`。

用法：
    python3 scripts/generate_sbom.py [--output PATH] [--check] [--strict]

默认输出到 stdout（JSON）。`--check` 生成到临时文件并验证结构（供 CI 门禁使用），
不写任何可复现路径到 stdout 之外的持久位置。
"""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import tempfile
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "src-tauri" / "Cargo.toml"
FRONTEND_LOCKFILE = ROOT / "frontend" / "package-lock.json"
APP_PACKAGE = "chinese-copywriting-formatter"
BOM_FORMAT = "CycloneDX"
BOM_VERSION = 1
BOM_SPEC_VERSION = "1.5"


def fmt(ts: datetime) -> str:
    return ts.astimezone(timezone.utc).isoformat().replace("+00:00", "Z")


def cargo_packages() -> list[dict]:
    cmd = ["cargo", "metadata", "--manifest-path", str(MANIFEST), "--locked", "--format-version", "1"]
    metadata = json.loads(subprocess.run(cmd, cwd=ROOT, check=True, capture_output=True, text=True).stdout)
    return metadata.get("packages", [])


def npm_packages() -> list[tuple[str, str, bool]]:
    lock = json.loads(FRONTEND_LOCKFILE.read_text(encoding="utf-8"))
    packages = lock.get("packages", {})
    root = packages.get("", {})
    dev_reqs = {str(k) for k in root.get("devDependencies", {})}
    rows: list[tuple[str, str, bool]] = []
    seen: set[tuple[str, str]] = set()
    for path, pkg in packages.items():
        if not path:
            continue
        name = path.rsplit("node_modules/", 1)[-1]
        version = pkg.get("version", "UNKNOWN")
        key = (name, version)
        if key in seen:
            continue
        seen.add(key)
        rows.append((name, version, name in dev_reqs))
    return rows


def build_doc() -> dict:
    components: list[dict] = []
    seen: set[tuple[str, str]] = set()
    for pkg in cargo_packages():
        name, version = pkg["name"], pkg.get("version", "")
        if name == APP_PACKAGE and pkg.get("source") is None:
            continue
        key = (name, version)
        if key in seen:
            continue
        seen.add(key)
        components.append(
            {
                "type": "library",
                "name": name,
                "version": version,
                "scope": "required",
                "properties": [{"name": "supplier", "value": "crates.io"}],
            }
        )
    for name, version, is_dev in npm_packages():
        key = (name, version)
        if key in seen:
            continue
        seen.add(key)
        props: list[dict] = []
        if is_dev:
            props.append({"name": "development", "value": "true"})
        components.append(
            {
                "type": "library",
                "name": name,
                "version": version,
                "scope": "optional" if is_dev else "required",
                "properties": props,
            }
        )
    return {
        "bomFormat": BOM_FORMAT,
        "specVersion": BOM_SPEC_VERSION,
        "version": BOM_VERSION,
        "metadata": {
            "timestamp": fmt(datetime.now(tz=timezone.utc)),
            "tools": [{"vendor": "CopyPolish", "name": "generate_sbom.py", "version": "1.0"}],
        },
        "components": components,
    }


def validate(doc: dict) -> None:
    if doc.get("bomFormat") != BOM_FORMAT or doc.get("specVersion") != BOM_SPEC_VERSION:
        raise ValueError("invalid CycloneDX header")
    components = doc.get("components")
    if not isinstance(components, list) or not components:
        raise ValueError("SBOM has no components")
    valid_types = {"library", "application", "framework", "container", "file", "device"}
    for c in components:
        if c.get("type") not in valid_types:
            raise ValueError(f"unexpected component type: {c.get('type')}")
    scopes = {c.get("scope") for c in components}
    if not scopes <= {"required", "optional", "excluded"}:
        raise ValueError(f"unexpected component scope: {scopes}")
    names = {(c.get("name"), c.get("version")) for c in components}
    if len(names) != len(components):
        raise ValueError("duplicate component (name, version)")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", help="输出 JSON 路径（默认 stdout）")
    parser.add_argument("--check", action="store_true", help="生成到临时文件并校验结构，不持久化")
    args = parser.parse_args()

    doc = build_doc()
    validate(doc)

    if args.check:
        with tempfile.TemporaryDirectory(prefix="copypolish-sbom-") as tmp:
            out = Path(tmp) / "sbom.json"
            out.write_text(json.dumps(doc, ensure_ascii=False, indent=2), encoding="utf-8")
            recheck = json.loads(out.read_text(encoding="utf-8"))
            validate(recheck)
        print(f"SBOM check OK: {len(doc['components'])} components")
        return 0

    blob = json.dumps(doc, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        Path(args.output).write_text(blob, encoding="utf-8")
        print(f"SBOM written: {args.output}")
    else:
        print(blob)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())