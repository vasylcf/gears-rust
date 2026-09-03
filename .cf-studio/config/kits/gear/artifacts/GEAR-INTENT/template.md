# GEAR-INTENT — {Request Name}

**ID**: `cpt-{system}-intent-{slug}`

**Date**: {YYYY-MM-DD}
**Requester**: {who asked, or the issue/task reference}

## Request

{The need in the requester's own terms: the problem or capability, who needs it, and the acceptance outcome. 1-3 paragraphs. Link the originating issue/task if one exists.}

## Classification

**Class**: {new gear | plugin | docs-only spec | small fix}

{Justification in 2-4 sentences: why this class and not the neighbors. A request touching public contracts is never a small fix. A capability an existing gear should own is a plugin or feature, not a new gear.}

## Similar Gears

{Result of the catalog lookup (`find_similar_gears.py`). List the closest existing gears with one line each on why they do or do not cover the need.}

| Gear | Tier / status | Overlap | Verdict |
|------|---------------|---------|---------|
| {gear} | {tier, status} | {what overlaps} | {reuse | extend | not a match — why} |

**Reuse-or-build call**: {reuse existing gear X | extend gear X with a plugin/feature | build new — no existing gear owns this domain}

## Reuse Profile

**Archetype**: {REST+DB service | REST-only | stateful | system | plugin | SDK-only | out-of-process gRPC}

**Reference gear to imitate**: {gear name and why}

**Core toolkit libraries**: {from the archetype profile}

**Optional toolkit libraries**: {only those this request actually needs, with reason}

**Platform dependencies**: {runtime gear deps, e.g. authz-resolver, types-registry}

## Document Set

{Which specification documents this request needs and why — full set for a new gear; reuse of host specs for plugins; none for a small fix (scope goes to the issue/PR description).}

| Document | Needed | Reason |
|----------|--------|--------|
| GEAR-PRD | {yes/no} | {reason} |
| GEAR-DESIGN | {yes/no} | {reason} |
| GEAR-ADR | {as decisions arise} | {expected decision areas} |
| GEAR-DECOMPOSITION | {yes/no} | {reason} |
| GEAR-FEATURE | {optional} | {only for complex behaviors} |

## Route Decision

**Route**: {the confirmed workflow sequence, e.g. doc-prd → doc-design → decompose → scaffold → implement → pr}

**First step**: {the exact next workflow to invoke and its input}

**Confirmed by**: {requester/contributor}, {date}
