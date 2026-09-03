#!/usr/bin/env python3
"""Deterministic check: every decomposition phase carries a runnable gate,
and (optionally) a phase report covers every gate of its phase.

Stdlib-only.

Usage:
    check_phase_gates.py DECOMPOSITION.md            # phases all carry gates
    check_phase_gates.py DECOMPOSITION.md --report GEAR-IMPL-REPORT.md
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

PHASE_ID_RE = re.compile(r"\*\*ID\*\*:\s*`(cpt-[a-z0-9-]+-phase-[a-z0-9-]+)`")
REPORT_PHASE_RE = re.compile(r"\*\*Phase\*\*:\s*`(cpt-[a-z0-9-]+-phase-[a-z0-9-]+)`")


def parse_phases(text: str) -> list[dict]:
    """Split on H3 headings; collect phase id and fenced gate commands."""
    phases = []
    sections = re.split(r"^### ", text, flags=re.MULTILINE)[1:]
    for sec in sections:
        pid = PHASE_ID_RE.search(sec)
        gate_block = re.search(r"^#### Gate\s*\n+```(?:sh|bash)?\n(.*?)\n```", sec, re.DOTALL | re.MULTILINE)
        commands = []
        if gate_block:
            for line in gate_block.group(1).splitlines():
                line = line.strip()
                if not line or line.startswith("#"):
                    continue
                # a `a && b` chain counts as separate commands the report must cover
                commands.extend(part.strip() for part in line.split("&&") if part.strip())
        phases.append(
            {
                "title": sec.splitlines()[0].strip(),
                "id": pid.group(1) if pid else None,
                "commands": commands,
            }
        )
    return phases


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("decomposition", type=Path)
    ap.add_argument("--report", type=Path, help="phase report to audit against its phase's gates")
    args = ap.parse_args()

    text = args.decomposition.read_text(encoding="utf-8")
    phases = parse_phases(text)
    failures = []

    if not phases:
        failures.append("no phases found (expected H3 phase sections)")
    for ph in phases:
        if not ph["id"]:
            failures.append(f"phase '{ph['title']}' has no cpt phase ID")
        if not ph["commands"]:
            failures.append(f"phase '{ph['title']}' has no runnable gate command")

    if args.report:
        rtext = args.report.read_text(encoding="utf-8")
        rphase = REPORT_PHASE_RE.search(rtext)
        if not rphase:
            failures.append("report: no phase reference found")
        else:
            target = next((p for p in phases if p["id"] == rphase.group(1)), None)
            if target is None:
                failures.append(f"report: phase {rphase.group(1)} not found in decomposition")
            else:
                for cmd in target["commands"]:
                    # each gate command (or its head token sequence) must appear in the report
                    if cmd not in rtext:
                        failures.append(f"report: gate command not covered: {cmd}")
                if re.search(r"\|\s*`[^`]+`\s*\|\s*(fail|FAIL)", rtext):
                    failures.append("report: a gate row records a failing outcome")

    if failures:
        for f in failures:
            print(f"FAIL {f}")
        return 1
    print(f"check_phase_gates: {len(phases)} phase(s) OK" + (", report covered" if args.report else ""))
    return 0


if __name__ == "__main__":
    sys.exit(main())
