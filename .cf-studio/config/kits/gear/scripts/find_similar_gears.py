#!/usr/bin/env python3
"""Similar-gear lookup over the gear catalog (intake deterministic step).

Scores existing gears against requested capabilities and keywords, prints the
top matches with tier/status/deps, and suggests the archetype profile.

Stdlib-only.

Usage:
    find_similar_gears.py --catalog data/gear-catalog.json \
        --capabilities rest,db --keywords audit,event,trail [--top 5] [--json]
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ARCHETYPES = [
    # (name, required caps subset, reference gear)
    ("REST+DB service gear", {"rest", "db"}, "simple-user-settings / users-info"),
    ("stateful gear", {"stateful"}, "credstore / usage-collector"),
    ("REST-only gear", {"rest"}, "file-parser / nodes-registry"),
    ("system gear", {"system"}, "tenant-resolver / types-registry"),
    ("out-of-process gRPC gear", {"grpc"}, "calculator + calculator-gateway"),
]


def score(gear: dict, caps: set[str], keywords: list[str]) -> float:
    s = 0.0
    gcaps = set(gear.get("capabilities") or [])
    if caps:
        s += 2.0 * len(caps & gcaps) / len(caps)
    text = " ".join(
        [gear["id"], gear.get("tier", ""), " ".join(gear.get("gear_deps") or [])]
    ).lower()
    for kw in keywords:
        if kw and kw.lower() in text:
            s += 1.0
    # implemented gears are better imitation targets
    if gear.get("status") == "implemented":
        s += 0.5
    return s


def pick_archetype(caps: set[str]) -> tuple[str, str]:
    for name, required, ref in ARCHETYPES:
        if required <= caps:
            return name, ref
    return ("SDK-only / to be decided", "llm-gateway-sdk")


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--catalog", required=True, type=Path)
    ap.add_argument("--capabilities", default="", help="comma-separated: rest,db,stateful,system,grpc,...")
    ap.add_argument("--keywords", default="", help="comma-separated domain keywords")
    ap.add_argument("--top", type=int, default=5)
    ap.add_argument("--json", action="store_true", dest="as_json")
    args = ap.parse_args()

    catalog = json.loads(args.catalog.read_text(encoding="utf-8"))
    caps = {c.strip() for c in args.capabilities.split(",") if c.strip()}
    keywords = [k.strip() for k in args.keywords.split(",") if k.strip()]
    if not caps and not keywords:
        print("error: provide --capabilities and/or --keywords", file=sys.stderr)
        return 2

    ranked = sorted(
        (g for g in catalog["gears"] if g.get("tier") != "example"),
        key=lambda g: score(g, caps, keywords),
        reverse=True,
    )
    top = [g for g in ranked if score(g, caps, keywords) > 0][: args.top]
    archetype, reference = pick_archetype(caps)

    result = {
        "query": {"capabilities": sorted(caps), "keywords": keywords},
        "catalog_snapshot": catalog.get("generated"),
        "archetype": archetype,
        "reference_gears": reference,
        "matches": [
            {
                "gear": g["id"],
                "tier": g["tier"],
                "status": g["status"],
                "capabilities": g.get("capabilities") or [],
                "runtime_deps": g.get("gear_deps") or [],
                "score": round(score(g, caps, keywords), 2),
            }
            for g in top
        ],
    }
    if args.as_json:
        print(json.dumps(result, indent=2))
    else:
        print(f"catalog snapshot: {result['catalog_snapshot']}")
        print(f"archetype: {archetype}  (imitate: {reference})")
        if not top:
            print("no matches for query:", result["query"])
        for m in result["matches"]:
            print(
                f"  {m['score']:>4}  {m['gear']:<32} {m['tier']}/{m['status']}"
                f"  caps={','.join(m['capabilities']) or '-'}"
                f"  deps={','.join(m['runtime_deps']) or '-'}"
            )
    return 0


if __name__ == "__main__":
    sys.exit(main())
