# GEAR-DECOMPOSITION Checklist

## Prerequisites

- GEAR-PRD and GEAR-DESIGN passed their deterministic gates.

## Applicability Context

Applies to the initial decomposition and to re-planning during implementation (a changed plan re-runs this checklist).

## Severity Dictionary

| Severity | Meaning |
|----------|---------|
| CRITICAL | The plan cannot keep the workspace green; block |
| HIGH | Materially weakens deliverability; fix before implementation |
| MEDIUM | Quality gap; fix or justify |
| LOW | Style advice |

## MUST HAVE

- `DEC-001` (CRITICAL) — Every phase has Scope, Completion Criteria, and a runnable Gate command.
- `DEC-002` (CRITICAL) — Completion criteria are provable by the named gate.
- `DEC-003` (HIGH) — Phase order follows dependency reality (SDK before host; storage before features that need it).
- `DEC-004` (HIGH) — Every p1 requirement is covered by some phase's scope; coverage is by ID.
- `DEC-005` (MEDIUM) — Each phase is one reviewable increment (roughly one PR at the reviewable ceiling or below).
- `DEC-006` (MEDIUM) — The final phase's gate includes the full local gate and the end-to-end run.
- `DEC-007` (LOW) — Deferred work is named in the overview, not silently missing.

## MUST NOT HAVE

- Phases without gates; gates that cannot run locally.
- Behavior specifications (route to GEAR-FEATURE).
- A final phase that skips the end-to-end run.
