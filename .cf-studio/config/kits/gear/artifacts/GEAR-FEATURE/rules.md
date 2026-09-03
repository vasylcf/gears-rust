# GEAR-FEATURE Rules

**Artifact**: GEAR-FEATURE
**Kit**: gear

```pdsl
UNIT GearFeatureAuthoring

PURPOSE:
  Specify one complex behavior with flows, a testable definition of done, and scenarios.

WHEN:
  - REQUIRE specifying behavior too complex for GEAR-DESIGN prose (states, algorithms, multi-step flows)

DO:
  - LOAD {gear_feature_template} for structure
  - SET ID = cpt-{hierarchy-prefix}-feature-{slug}; filename NNNN-{cpt-id}.md in the gear's docs/features/
  - RUN map the Summary to the implementing fr IDs and the delivering decomposition phase
  - RUN derive test scenarios from the definition of done — every DoD item has at least one scenario

RULES:
  - ALWAYS treat FEATURE as optional: most gear behavior lives in GEAR-DESIGN; write a FEATURE only when flows/states/algorithms need their own contract
  - ALWAYS include failure and edge behavior in Behavior, not only the happy path
  - ALWAYS make every definition-of-done item independently testable
  - NEVER leave placeholders (TODO, TBD, FIXME)
```

```pdsl
UNIT GearFeatureOmissions

PURPOSE:
  Enforce GEAR-FEATURE scope boundaries. Report as a violation if found.

RULES:
  - NEVER restate requirements (GEAR-FEA-NO-001, MEDIUM) — reference fr IDs
  - NEVER specify architecture or storage (GEAR-FEA-NO-002, HIGH) — belongs in GEAR-DESIGN
  - NEVER write a definition of done no test can verify (GEAR-FEA-NO-003, CRITICAL)
```

```pdsl
UNIT GearFeatureValidate

PURPOSE:
  Run deterministic validation on the GEAR-FEATURE.

DO:
  - RUN cfs toc <artifact-file>
  - RUN cfs validate --artifact <path> — must report PASS
  - RUN cfs validate-toc <artifact-file> — must report PASS
  - RUN cfs check-language <artifact-file> — must report PASS
```
