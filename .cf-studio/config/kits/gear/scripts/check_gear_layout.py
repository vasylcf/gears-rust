#!/usr/bin/env python3
"""Deterministic check of a gear's canonical layout and registration surface
in a gears-rust working copy (the scaffold gate's kit-owned part).

Checks (required unless noted):
  - kebab-case gear name
  - gear directory with <name>/ (impl) and, unless --plugin, <name>-sdk/ crates
  - impl crate declares #[toolkit::gear]
  - workspace Cargo.toml lists both crates as members (path match)
  - example server registers the gear: feature in apps Cargo.toml + use in registered_gears.rs
  - cargo-shear ignore list mentions the crates (warning only — dep-driven)
  - docs/PRD.md and docs/DESIGN.md exist (warning for plugins)

Stdlib-only.

Usage:
    check_gear_layout.py --repo ../gears-rust --gear audit-log [--plugin]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

KEBAB = re.compile(r"^[a-z][a-z0-9]*(-[a-z0-9]+)*$")


def find_gear_dir(repo: Path, gear: str) -> Path | None:
    candidates = [
        *repo.glob(f"gears/{gear}"),
        *repo.glob(f"gears/*/{gear}"),
        *repo.glob(f"gears/*/plugins/{gear}"),
        *repo.glob(f"gears/*/*/plugins/{gear}"),
    ]
    return candidates[0] if candidates else None


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--repo", required=True, type=Path)
    ap.add_argument("--gear", required=True)
    ap.add_argument("--plugin", action="store_true", help="single-crate plugin layout")
    args = ap.parse_args()
    repo, gear = args.repo.resolve(), args.gear
    errors: list[str] = []
    warnings: list[str] = []

    if not KEBAB.match(gear):
        errors.append(f"gear name '{gear}' is not kebab-case")

    gdir = find_gear_dir(repo, gear)
    if gdir is None:
        print(f"FAIL gear directory for '{gear}' not found under gears/")
        return 1

    impl = gdir / gear if (gdir / gear).is_dir() else gdir
    sdk = gdir / f"{gear}-sdk"
    if not (impl / "Cargo.toml").is_file():
        errors.append(f"impl crate missing: {impl}/Cargo.toml")
    if not args.plugin and not (sdk / "Cargo.toml").is_file():
        errors.append(f"sdk crate missing: {sdk}/Cargo.toml")

    macro_found = any(
        re.search(r"#\[(?:toolkit::gear|modkit::module)\s*\(", p.read_text(encoding="utf-8", errors="replace"))
        for p in impl.rglob("*.rs")
    )
    if not macro_found:
        errors.append("no #[toolkit::gear] declaration found in the impl crate")

    ws = repo / "Cargo.toml"
    ws_text = ws.read_text(encoding="utf-8") if ws.is_file() else ""
    rel_impl = str(impl.relative_to(repo)).replace("\\", "/")
    if rel_impl not in ws_text:
        errors.append(f"workspace Cargo.toml does not list member '{rel_impl}'")
    if not args.plugin:
        rel_sdk = str(sdk.relative_to(repo)).replace("\\", "/")
        if rel_sdk not in ws_text:
            errors.append(f"workspace Cargo.toml does not list member '{rel_sdk}'")

    apps_manifest = repo / "apps/cf-gears-example-server/Cargo.toml"
    reg_file = repo / "apps/cf-gears-example-server/src/registered_gears.rs"
    snake = gear.replace("-", "_")
    if apps_manifest.is_file() and gear not in apps_manifest.read_text(encoding="utf-8"):
        errors.append(f"example server Cargo.toml has no feature/dependency for '{gear}'")
    if reg_file.is_file() and snake not in reg_file.read_text(encoding="utf-8"):
        errors.append(f"registered_gears.rs has no 'use {snake} as _;' registration")

    if "cargo-shear" in ws_text:
        shear_section = ws_text.split("cargo-shear", 1)[1][:4000]
        if snake not in shear_section and gear not in shear_section:
            warnings.append(
                "cargo-shear ignore list does not mention the gear — required once the gear "
                "is a dependency of the example server (nightly shear breaks main otherwise)"
            )

    docs = gdir / "docs"
    for doc in ("PRD.md", "DESIGN.md"):
        if not (docs / doc).is_file():
            (warnings if args.plugin else errors).append(f"docs/{doc} missing")

    for w in warnings:
        print(f"WARN {w}")
    if errors:
        for e in errors:
            print(f"FAIL {e}")
        return 1
    print(f"check_gear_layout: '{gear}' OK ({len(warnings)} warning(s))")
    return 0


if __name__ == "__main__":
    sys.exit(main())
