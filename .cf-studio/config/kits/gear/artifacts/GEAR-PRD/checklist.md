# GEAR-PRD Checklist

## Prerequisites

- A confirmed GEAR-INTENT exists and is linked.
- The ID prefix (system slug) is registered in the project artifact config.

## Applicability Context

Applies to the PRD of a new gear and to material PRD revisions of an existing gear. Review altitude is WHAT-level only: flag missing or contradictory requirements, not implementation choices.

## Severity Dictionary

| Severity | Meaning |
|----------|---------|
| CRITICAL | Downstream design would have to guess; block progression |
| HIGH | Materially incomplete or ambiguous; fix before design |
| MEDIUM | Quality gap; fix or justify |
| LOW | Style/completeness advice |

## MUST HAVE

- `PRD-001` (CRITICAL) — Every functional requirement is specific, verifiable, and carries an ID with priority.
- `PRD-002` (CRITICAL) — Gear Classification matches the confirmed intent (class, capabilities, archetype).
- `PRD-003` (CRITICAL) — Boundaries against adjacent gears are explicit: every overlapping capability names the owning gear (in scope here, or out of scope with owner).
- `PRD-004` (HIGH) — Authorization stated as per-actor/operation permissions for every mutating capability.
- `PRD-005` (HIGH) — Public Interfaces states who consumes the SDK and which GTS extension points exist, without contract detail.
- `PRD-006` (HIGH) — NFRs are measurable with thresholds; platform defaults are not restated.
- `PRD-007` (MEDIUM) — Use cases cover the primary actor journeys, including one failure/refusal path.
- `PRD-008` (MEDIUM) — Acceptance criteria are business-level and testable.
- `PRD-009` (LOW) — Risks include at least one adoption/integration risk, not only technical ones.

## MUST NOT HAVE

- Crate layout, endpoints, schemas, decision trade-off analysis, implementation phases (route to GEAR-DESIGN / GEAR-ADR / GEAR-DECOMPOSITION).
- Unqualified "the gear must be secure/fast/reliable" statements.
- Requirements for capabilities the classification placed out of scope.
