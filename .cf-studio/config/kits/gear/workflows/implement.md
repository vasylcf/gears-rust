---
cf-studio: true
type: workflow
name: cf-gear-implement
description: Invoke when the user asks to implement a gear in gears-rust from its approved decomposition — e.g. "implement audit-log", "run phase 2", "continue the implementation". Executes GEAR-DECOMPOSITION phase by phase through the core cf-coding engine: gear-scoped gate after every phase, a GEAR-IMPL-REPORT per phase close, full local gate plus end-to-end at completion. Never advances past a failing gate.
version: 1.0
purpose: Phase-by-phase gear implementation over cf-coding — per-phase gear-scoped gates, GEAR-IMPL-REPORT close-outs, full gate plus e2e at completion
---

# cf-gear-implement — Phased gear implementation

Preset over the core `cf-coding` engine with a phase loop around it: each
approved GEAR-DECOMPOSITION phase is implemented through cf-coding, closed by
its declared gate, and recorded as a GEAR-IMPL-REPORT before the next phase
begins. Authors no code in the preset itself.

## Route

Phase loop: one confirmation gate per phase close (present the report, confirm continuation); coding-step gates inside a phase are owned by the core cf-coding engine. Announce the phase plan (count and current position) at start and per the SKILL.md progress protocol.

## Prerequisite Checklist

- [ ] The gear's GEAR-DECOMPOSITION is approved and passes `{scripts}/check_phase_gates.py`
- [ ] The scaffold gate passed (`cf-gear-scaffold`), or the gear already exists
- [ ] The workspace builds clean at the starting tip

```pdsl
UNIT GearImplementPreset
PURPOSE: Execute the approved GEAR-DECOMPOSITION phase by phase through the core cf-coding workflow, gating and reporting every phase.
STATE:
  SET ARTIFACT_KIND: CODE (default CODE, scope workflow_run)
  SET CURRENT_PHASE: the first unclosed phase of the decomposition (scope workflow_run)
DO:
  SET ARTIFACT_KIND = CODE
  RUN resolve the gear's approved GEAR-DECOMPOSITION and the accumulated GEAR-IMPL-REPORTs; CURRENT_PHASE = the first phase without a passing report
  RUN announce the phase plan: total phases, closed phases, CURRENT_PHASE scope and gate
  LOAD {cf-studio-path}/.core/workflows/coding.md as the controlling implementation workflow
  CONTINUE CodingBootstrap
RULES:
  ALWAYS scope every coder dispatch to CURRENT_PHASE only: its Scope items, mapped fr/component IDs, and the linked GEAR-FEATURE when one exists
  ALWAYS inject the embedded GearImplementationRules unit as additional implementation rules into every coder dispatch
  ALWAYS set the deterministic gate target for the phase to its declared Gate commands (gear-scoped: `make gear-ci GEAR=<name>`; final phase: the full local gate plus the end-to-end run) in addition to cf-coding's own test/lint/build gates
  ALWAYS open a phase with the cheap workspace gate `cargo check --workspace --all-targets` before the first coder dispatch — it catches cross-gear breakage from shared seams minutes before the phase gate would
  ALWAYS extend the close gate of a storage-touching phase (migrations, entities, repositories) of a gear with the `db` capability with the gear's Docker DB lane (`make test-<gear>-db`: testcontainers suites on the pinned PostgreSQL and MySQL images); SQLite-only test evidence never closes such a phase — backend-specific DDL and driver encodings fail only on the real backends
  ALWAYS close a phase by authoring its GEAR-IMPL-REPORT from {gear_impl_report_template} per {gear_impl_report_rules}, validating it with `cfs validate --artifact <report>` and `python3 {scripts}/check_phase_gates.py <decomposition> --report <report>`
  ALWAYS present the phase report and confirm continuation before starting the next phase (user gate — confirmation)
  ALWAYS reroute material plan deviations through cf-gear-decompose before continuing; record the deviation in the report
  NEVER advance past a failing gate; fix and re-run until PASS — recovery never asks
  NEVER author code in this preset; delegate all implementation and review to cf-coding
NOTES:
  When no GEAR-FEATURE exists, the implementation contract is the phase Scope plus the linked GEAR-DESIGN and GEAR-PRD IDs — mirroring direct-from-design coding; @cpt-* code traceability markers are used when the host project's traceability mode is FULL.
```

```pdsl
UNIT GearImplementationRules
PURPOSE: Implement one decomposition phase with TDD, platform invariants, and traceability.
WHEN:
  REQUIRE implementing or revising gear code for a decomposition phase
DO:
  RUN split CURRENT_PHASE's scope into slices ordered by dependency, each independently testable
  RUN identify risky slices touching privilege boundaries, Secure ORM, SecurityContext, secrets, FIPS behavior, or registry/autodetect logic; record review evidence for them
  RUN implement one slice at a time with TDD: failing test first, smallest passing code, then refactor
  RUN after each slice, run the project's tests, lint, and build for the touched crates; fix every finding before the next slice
RULES:
  ALWAYS treat the phase Scope plus the linked GEAR-DESIGN/GEAR-PRD (and GEAR-FEATURE when present) IDs as the implementation contract; never broaden scope without an upstream artifact change
  ALWAYS preserve SDK-first public contracts, domain/API/infrastructure separation, secure-ORM-only data access, canonical error behavior, and SecurityContext propagation unless the design documents an approved deviation
  ALWAYS keep repository traits speaking domain-owned record types, mapping SeaORM entities to them only inside the infrastructure repository — the entity-through-the-trait shortcut compiles clean and fails only at the architecture lint (DE0301)
  ALWAYS keep tests in sibling *_test.rs files per the host convention; add compile-fail tests when the change exposes compile-time guarantees and a harness exists
  ALWAYS keep {codebase_checklist} review-only when the host project provides one; NEVER load it during generation
  ALWAYS keep slice scope small enough that tests, code, and markers review together
  NEVER introduce orphan, duplicate, or speculative @cpt-* markers; generate marker IDs only from existing artifact IDs
  NEVER leave deterministic validation, tests, lint, or build failures unresolved when the commands are available
```

## Next Steps

- After the final phase's report: `cf-gear-pr` to assemble the review-ready pull request
