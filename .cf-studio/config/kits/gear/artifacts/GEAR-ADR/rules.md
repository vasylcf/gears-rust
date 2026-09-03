# GEAR-ADR Rules

**Artifact**: GEAR-ADR
**Kit**: gear

```pdsl
UNIT GearAdrAuthoring

PURPOSE:
  Record one gear architecture decision with real options and confirmed consequences.

WHEN:
  - REQUIRE recording or revising a gear architecture decision

DO:
  - LOAD {gear_adr_template} for structure
  - SET ID = cpt-{hierarchy-prefix}-adr-{slug}; filename NNNN-{cpt-id}.md in the gear's docs/ADR/
  - RUN add the ADR ID to the gear DESIGN's Architecture Drivers in the same change
  - RUN attach evidence (SPIKE, alternatives evaluation) for contested decisions; mark superseded evidence as dated, keep it

RULES:
  - ALWAYS record at least two genuinely considered options — a one-option ADR is a violation
  - ALWAYS state a concrete confirmation mechanism (lint, test, CI check, review checkpoint)
  - ALWAYS keep one decision per ADR; a second dilemma gets its own record
  - ALWAYS place the ADR in the gear it governs; decisions about another gear move there
  - NEVER leave placeholders (TODO, TBD, FIXME)
```

```pdsl
UNIT GearAdrOmissions

PURPOSE:
  Enforce GEAR-ADR scope boundaries. Report as a violation if found.

RULES:
  - NEVER record requirements as decisions (GEAR-ADR-NO-001, HIGH) — requirements belong in GEAR-PRD
  - NEVER record implementation detail without a dilemma (GEAR-ADR-NO-002, MEDIUM) — plain design belongs in GEAR-DESIGN
  - NEVER present rejected options as strawmen (GEAR-ADR-NO-003, HIGH) — each option carries its genuine pros
  - NEVER leave a decision unreferenced from the gear DESIGN (GEAR-ADR-NO-004, CRITICAL)
```

```pdsl
UNIT GearAdrValidate

PURPOSE:
  Run deterministic validation on the GEAR-ADR.

DO:
  - RUN cfs toc <artifact-file>
  - RUN cfs validate --artifact <path> — must report PASS (includes DESIGN reference coverage)
  - RUN cfs validate-toc <artifact-file> — must report PASS
  - RUN cfs check-language <artifact-file> — must report PASS
```
