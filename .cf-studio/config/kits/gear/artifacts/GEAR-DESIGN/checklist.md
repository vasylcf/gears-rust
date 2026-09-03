# GEAR-DESIGN Checklist

## Prerequisites

- The gear's GEAR-PRD passed its deterministic gate.
- The canonical layout reference from the host repository was read.

## Applicability Context

Applies to a new gear's design and to revisions that change components, contracts, or storage. Review altitude: HOW the requirements are realized; requirement changes route back to the PRD.

## Severity Dictionary

| Severity | Meaning |
|----------|---------|
| CRITICAL | Violates a platform invariant or leaves requirements unallocated; block progression |
| HIGH | Materially incomplete allocation or missing seam plan; fix before decomposition |
| MEDIUM | Quality gap; fix or justify |
| LOW | Style/completeness advice |

## MUST HAVE

- `DSN-001` (CRITICAL) — Every PRD fr and nfr appears in the driver tables with a concrete design response.
- `DSN-002` (CRITICAL) — Gear Structure declares crates, macro capabilities/deps, and the full registration surface.
- `DSN-003` (CRITICAL) — All persistent data is tenant-scoped through the secure ORM; scoping columns named per table.
- `DSN-004` (HIGH) — Every GTS type the gear declares or extends appears with its ID and declaration site.
- `DSN-005` (HIGH) — Every plugin seam names its default/noop implementation.
- `DSN-006` (HIGH) — Every ADR recorded for this gear is listed in Architecture Drivers.
- `DSN-007` (MEDIUM) — Components state boundaries (what they do NOT do), not only scope.
- `DSN-008` (MEDIUM) — At least one sequence covers the primary use case end to end.
- `DSN-009` (LOW) — Dependency table distinguishes gear deps from toolkit libraries.

## MUST NOT HAVE

- Inline trade-off analysis (route to GEAR-ADR); requirement prose restated from the PRD.
- Embedded DDL / OpenAPI / GTS schema bodies.
- Raw database access paths bypassing the secure ORM.
