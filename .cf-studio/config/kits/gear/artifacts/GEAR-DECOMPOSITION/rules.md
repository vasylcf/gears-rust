# GEAR-DECOMPOSITION Rules

**Artifact**: GEAR-DECOMPOSITION
**Kit**: gear

```pdsl
UNIT GearDecompositionAuthoring

PURPOSE:
  Decompose an approved gear design into phases that each end buildable, runnable, and tested.

WHEN:
  - REQUIRE decomposing a gear design into implementation phases

DO:
  - LOAD {gear_decomposition_template} for structure
  - LOAD the gear's GEAR-DESIGN and GEAR-PRD; map every phase's scope to fr and component IDs
  - SET phase IDs = cpt-{hierarchy-prefix}-phase-{slug}; order phases by the golden path: SDK contract first, gear host and registration next, seams and features after
  - SET every phase's gate to a real repository command (gear-scoped check per phase; full gate plus e2e for the final phase)
  - RUN size phases so one phase is one reviewable increment

RULES:
  - ALWAYS give every phase all three subsections: Scope, Completion Criteria, Gate
  - ALWAYS make completion criteria verifiable by the named gate — a criterion no command can prove is a violation
  - ALWAYS keep the workspace green between phases: no phase may end with failing builds or tests
  - ALWAYS cover every p1 requirement in some phase's scope
  - NEVER leave placeholders (TODO, TBD, FIXME)
```

```pdsl
UNIT GearDecompositionOmissions

PURPOSE:
  Enforce GEAR-DECOMPOSITION scope boundaries. Report as a violation if found.

RULES:
  - NEVER write feature-level behavior specs in a phase (GEAR-DEC-NO-001, MEDIUM) — belongs in GEAR-FEATURE
  - NEVER define a phase without a deterministic gate command (GEAR-DEC-NO-002, CRITICAL)
  - NEVER plan a phase that leaves the workspace unbuildable (GEAR-DEC-NO-003, CRITICAL)
  - NEVER bundle unrelated deliverables to skip a gate (GEAR-DEC-NO-004, HIGH)
```

```pdsl
UNIT GearDecompositionValidate

PURPOSE:
  Run deterministic validation on the GEAR-DECOMPOSITION.

DO:
  - RUN cfs toc <artifact-file>
  - RUN cfs validate --artifact <path> — must report PASS
  - RUN cfs validate-toc <artifact-file> — must report PASS
  - RUN cfs check-language <artifact-file> — must report PASS
  - RUN python3 {scripts}/check_phase_gates.py <artifact-file> — every phase carries a non-empty gate command
```
