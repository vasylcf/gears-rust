# GEAR-IMPL-REPORT Checklist

## Prerequisites

- The referenced phase exists in the gear's GEAR-DECOMPOSITION.
- All of the phase's gate commands have been executed.

## Applicability Context

Applies at every phase close. The report is the auditable record reviewers and the PR digest rely on.

## Severity Dictionary

| Severity | Meaning |
|----------|---------|
| CRITICAL | The report misrepresents verification; block |
| HIGH | Materially incomplete record; fix before the next phase |
| MEDIUM | Quality gap; fix or justify |
| LOW | Style advice |

## MUST HAVE

- `RPT-001` (CRITICAL) — Every gate command of the phase appears with a pass outcome and evidence.
- `RPT-002` (CRITICAL) — The phase reference matches an existing decomposition phase ID.
- `RPT-003` (HIGH) — Work summary items map to the phase's scope items.
- `RPT-004` (HIGH) — Deviations are stated ("None" counts) and material ones name the re-planning follow-up.
- `RPT-005` (LOW) — Next phase names a real phase ID or declares delivery complete.

## MUST NOT HAVE

- Gate outcomes without evidence of execution.
- Silent scope drift: work items outside the phase's scope with no deviation entry.
- Forward planning beyond naming the next phase.
