# GEAR-INTENT Checklist

## Prerequisites

- The gear catalog resource is installed and its snapshot date is known.
- The requester is identifiable (person, issue, or task reference).

## Applicability Context

Applies to every gear-creation request before any specification or code work. A revision of an existing intent re-runs the full checklist.

## Severity Dictionary

| Severity | Meaning |
|----------|---------|
| CRITICAL | Misroutes the whole delivery; intent must not be confirmed |
| HIGH | Materially weakens the decision; fix before confirmation |
| MEDIUM | Quality gap; fix or justify in the document |
| LOW | Style/completeness advice |

## MUST HAVE

- `INT-001` (CRITICAL) — The classification names one of the four classes and the justification addresses why the neighboring classes were rejected.
- `INT-002` (CRITICAL) — The similar-gears table shows the actual lookup output (or the query with an empty result), and every close match has a verdict.
- `INT-003` (CRITICAL) — The route decision is explicitly confirmed by the requester with a date.
- `INT-004` (HIGH) — The reuse profile names the archetype, the reference gear, and the core library set.
- `INT-005` (HIGH) — The document-set table is consistent with the classification (new gear ⇒ PRD, DESIGN, DECOMPOSITION at minimum).
- `INT-006` (MEDIUM) — The request section states an acceptance outcome, not only a problem.
- `INT-007` (LOW) — Originating issue or task is linked when one exists.

## MUST NOT HAVE

- Requirements catalogs, architecture, crate layouts, endpoint lists (belong downstream).
- A "small fix" classification for anything touching a public contract.
- A reuse-or-build call that contradicts the similar-gears verdicts without explanation.
