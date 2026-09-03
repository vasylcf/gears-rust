# GEAR-IMPL-REPORT — {Gear Name} Phase {N}

**ID**: `cpt-{system}-report-{slug}`

**Date**: {YYYY-MM-DD}

<!-- Written at the close of every implementation phase. The accumulated reports
     feed the PR body digest. Gate results must be real command executions. -->

## Phase Reference

**Phase**: `cpt-{system}-phase-{slug}` from [DECOMPOSITION.md]({path})

## Work Summary

{What was built, changed, and tested in this phase. Bullet list of concrete deliverables (crates, modules, endpoints, entities, tests), each mapped to the scope items it fulfills.}

## Gate Results

| Gate command | Outcome | Evidence |
|--------------|---------|----------|
| `{command}` | {pass} | {run reference: log path, CI link, or timestamp} |

{Every command from the phase's Gate section appears here. A failed gate means the phase is not closed — the report is written only when all gates pass.}

## Deviations

{Departures from the decomposition (scope moved, criteria adjusted, order changed) with reasons — or the single word "None". A material deviation reroutes through the decomposition workflow before the next phase.}

## Next Phase

{The next phase ID to execute — or "Delivery complete", with the PR-preparation workflow as the next step.}
