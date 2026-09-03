---
status: accepted
date: 2026-08-22
decision-makers: audit-log maintainers
---

# Retention Runs In-Gear as a Background Task, Not as an External Job


<!-- toc -->

- [Context and Problem Statement](#context-and-problem-statement)
- [Decision Drivers](#decision-drivers)
- [Considered Options](#considered-options)
- [Decision Outcome](#decision-outcome)
  - [Consequences](#consequences)
  - [Confirmation](#confirmation)
- [Pros and Cons of the Options](#pros-and-cons-of-the-options)
  - [In-gear background task using the stateful capability](#in-gear-background-task-using-the-stateful-capability)
  - [External scheduled job calling a privileged deletion endpoint](#external-scheduled-job-calling-a-privileged-deletion-endpoint)
- [Traceability](#traceability)

<!-- /toc -->

**ID**: `cpt-cf-audit-log-adr-retention-enforcement`
## Context and Problem Statement

Per-tenant retention must delete expired audit records provably and record the deletion itself. Should enforcement run inside the gear as a stateful background task, or outside it as a scheduled platform job calling a deletion API?

## Decision Drivers

* Deletions must be tenant-scoped through the secure ORM — no out-of-band data access.
* The deletion event must enter the trail through the same validated ingestion path as any event.
* Operational simplicity: no new deployment unit for the first release.

## Considered Options

* In-gear background task using the stateful capability
* External scheduled job calling a privileged deletion endpoint

## Decision Outcome

Chosen option: "In-gear background task using the stateful capability", because it keeps all data access inside the gear's scoped storage (driver 1), reuses the ingestion path for deletion events naturally (driver 2), and adds no deployment unit (driver 3).

### Consequences

* Good, because retention logic, policy seam, and storage stay in one reviewable place.
* Good, because the gear macro's stateful capability provides lifecycle and cancellation for free.
* Bad, because long deletion sweeps share the gear's runtime; mitigated by batched deletes with yield points and a per-sweep budget.

### Confirmation

Gear-scoped tests cover a full retention sweep including the emitted deletion event; the architecture lints confirm no storage access outside the secure ORM.

## Pros and Cons of the Options

### In-gear background task using the stateful capability

* Good, because scoped storage access and policy seam stay internal.
* Good, because no privileged external deletion API exists to secure.
* Bad, because sweep load shares the gear's runtime budget.

### External scheduled job calling a privileged deletion endpoint

* Good, because sweep load is isolated from the gear's serving path.
* Bad, because it requires a privileged deletion API — a second, dangerous public surface.
* Bad, because a new deployment unit must be operated and secured.

## Traceability

- **GEAR-PRD**: [PRD.md](../PRD.md)
- **GEAR-DESIGN**: [DESIGN.md](../DESIGN.md)

This decision directly addresses:

* `cpt-cf-audit-log-fr-enforce-retention` — defines where and how enforcement executes
* `cpt-cf-audit-log-component-retention` — constrains the component to an in-gear stateful task
