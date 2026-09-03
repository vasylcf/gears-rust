# GEAR-FEATURE Checklist

## Prerequisites

- The implementing requirement IDs exist in the gear's GEAR-PRD.
- The delivering phase exists in the gear's GEAR-DECOMPOSITION.

## Applicability Context

Applies only when a FEATURE is written at all — the artifact is optional and most behavior belongs in GEAR-DESIGN. Existence review: first confirm the behavior warrants its own contract.

## Severity Dictionary

| Severity | Meaning |
|----------|---------|
| CRITICAL | The spec cannot drive implementation or testing; block |
| HIGH | Materially incomplete behavior; fix before implementation |
| MEDIUM | Quality gap; fix or justify |
| LOW | Style advice |

## MUST HAVE

- `FEA-001` (CRITICAL) — Every definition-of-done item is independently testable and has at least one scenario.
- `FEA-002` (HIGH) — Behavior covers failure and edge cases, not only the happy path.
- `FEA-003` (HIGH) — Summary maps to fr IDs and names the delivering phase.
- `FEA-004` (MEDIUM) — State machines and multi-step flows are drawn (Mermaid), not only narrated.
- `FEA-005` (LOW) — Scenarios use the given/when/then form.

## MUST NOT HAVE

- Architecture, storage, or contract definitions (route to GEAR-DESIGN).
- Requirement prose restated from the PRD.
- A FEATURE for behavior simple enough for the design document (delete it, move the content).
