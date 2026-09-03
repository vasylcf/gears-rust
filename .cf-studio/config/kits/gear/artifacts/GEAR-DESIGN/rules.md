# GEAR-DESIGN Rules

**Artifact**: GEAR-DESIGN
**Kit**: gear

```pdsl
UNIT GearDesignAuthoring

PURPOSE:
  Author or revise a gear technical design allocating every PRD requirement to architecture.

WHEN:
  - REQUIRE authoring or revising a gear DESIGN

DO:
  - LOAD {gear_design_template} for structure
  - LOAD the gear's GEAR-PRD; build the Functional Drivers table covering every fr ID and the NFR Allocation covering every nfr ID
  - RUN read the canonical layout reference in the host repository (docs/toolkit_unified_system/02_gear_layout_and_sdk_pattern.md) before writing Gear Structure
  - SET component IDs = cpt-{hierarchy-prefix}-component-{slug}; sequence IDs = cpt-{hierarchy-prefix}-seq-{slug}
  - RUN list every gear ADR ID in Architecture Drivers as decisions are recorded
  - RUN cfs list-ids to verify ID uniqueness

RULES:
  - ALWAYS cover every PRD fr and nfr ID in the driver tables — uncovered requirements are validation errors
  - ALWAYS declare the full registration surface in Gear Structure
  - ALWAYS state storage explicitly, including the stateless case
  - ALWAYS reference schema and contract files instead of embedding their bodies
  - ALWAYS treat {gear_design_checklist} as the single source of semantic quality criteria
  - NEVER leave placeholders (TODO, TBD, FIXME); NEVER duplicate IDs
```

```pdsl
UNIT GearDesignOmissions

PURPOSE:
  Enforce GEAR-DESIGN scope boundaries. Report as a violation if found.

RULES:
  - NEVER restate requirements as design (GEAR-DSN-NO-001, HIGH) — reference fr IDs instead
  - NEVER record decision trade-off analysis inline (GEAR-DSN-NO-002, CRITICAL) — belongs in GEAR-ADR, referenced from Architecture Drivers
  - NEVER embed full DDL, OpenAPI bodies, or GTS schema bodies (GEAR-DSN-NO-003, MEDIUM) — reference the files
  - NEVER design raw database access around the secure ORM (GEAR-DSN-NO-004, CRITICAL) — scoped access is a platform invariant
  - NEVER leave a plugin seam without a default/noop implementation plan (GEAR-DSN-NO-005, HIGH)
```

```pdsl
UNIT GearDesignValidate

PURPOSE:
  Run deterministic validation on the GEAR-DESIGN.

DO:
  - RUN cfs toc <artifact-file>
  - RUN cfs validate --artifact <path> — must report PASS (includes fr/nfr coverage)
  - RUN cfs validate-toc <artifact-file> — must report PASS
  - RUN cfs check-language <artifact-file> — must report PASS
  - RUN gts-validator over the artifact when it names GTS identifiers — must report no violations
```
