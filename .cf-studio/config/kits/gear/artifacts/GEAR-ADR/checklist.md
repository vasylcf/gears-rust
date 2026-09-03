# GEAR-ADR Checklist

## Prerequisites

- The gear's GEAR-DESIGN exists (the ADR must be referenced from its Architecture Drivers).

## Applicability Context

Applies to every recorded gear decision. Review altitude: is this a real dilemma, honestly weighed, with a confirmable outcome.

## Severity Dictionary

| Severity | Meaning |
|----------|---------|
| CRITICAL | The record cannot function as a decision; block |
| HIGH | Materially weakens the decision record; fix before acceptance |
| MEDIUM | Quality gap; fix or justify |
| LOW | Style advice |

## MUST HAVE

- `ADR-001` (CRITICAL) — A real dilemma with at least two genuinely considered options.
- `ADR-002` (CRITICAL) — The decision outcome names the chosen option and justifies it against the stated drivers.
- `ADR-003` (HIGH) — Consequences include at least one honest negative with mitigation.
- `ADR-004` (HIGH) — Confirmation names a concrete mechanism, not "code review will catch it".
- `ADR-005` (HIGH) — Traceability lists the requirement and design element IDs the decision constrains.
- `ADR-006` (MEDIUM) — Contested decisions carry evidence (SPIKE with measurements, or a dated alternatives evaluation).
- `ADR-007` (LOW) — Frontmatter status and date are current.

## MUST NOT HAVE

- One-option records; strawman rejected options.
- Multiple decisions in one record.
- Decisions about another gear (move the record there).
