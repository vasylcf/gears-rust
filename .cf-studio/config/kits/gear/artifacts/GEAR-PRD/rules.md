# GEAR-PRD Rules

**Artifact**: GEAR-PRD
**Kit**: gear

```pdsl
UNIT GearPrdAuthoring

PURPOSE:
  Author or revise a gear PRD that follows the template and stays requirements-only.

WHEN:
  - REQUIRE authoring or revising a gear PRD

DO:
  - LOAD {gear_prd_template} for structure
  - LOAD the confirmed GEAR-INTENT as input; carry its classification into Gear Classification verbatim
  - RUN read project config for ID prefix from {cf-studio-path}/config/artifacts.toml
  - LOAD {gear_prd_example} for content-depth reference
  - SET actor IDs = cpt-{hierarchy-prefix}-actor-{slug}; FR IDs = cpt-{hierarchy-prefix}-fr-{slug}; priorities p1-p9 by business impact
  - RUN cfs list-ids to verify ID uniqueness

RULES:
  - ALWAYS follow {gear_prd_template} structure; all required sections present and non-empty
  - ALWAYS keep the PRD requirements-only: WHAT the gear does, never HOW
  - ALWAYS state authorization as per-actor/operation permissions, never the generic "requires auth"
  - ALWAYS name the owning gear for every out-of-scope capability that exists elsewhere
  - ALWAYS treat {gear_prd_checklist} as the single source of semantic quality criteria
  - NEVER leave placeholders (TODO, TBD, FIXME); NEVER duplicate IDs
```

```pdsl
UNIT GearPrdOmissions

PURPOSE:
  Enforce GEAR-PRD scope boundaries. Report as a violation if found.

RULES:
  - NEVER include crate layout, layers, or module structure (GEAR-PRD-NO-001, CRITICAL) — belongs in GEAR-DESIGN
  - NEVER include REST endpoint specs, methods, status codes (GEAR-PRD-NO-002, CRITICAL) — belongs in GEAR-DESIGN
  - NEVER include database schemas or GTS schema bodies (GEAR-PRD-NO-003, HIGH) — belongs in GEAR-DESIGN
  - NEVER include architecture decisions with options and trade-offs (GEAR-PRD-NO-004, CRITICAL) — belongs in GEAR-ADR
  - NEVER include implementation phases or tasks (GEAR-PRD-NO-005, HIGH) — belongs in GEAR-DECOMPOSITION
  - NEVER restate platform-default NFRs as gear NFRs (GEAR-PRD-NO-006, MEDIUM) — only deviations and extensions
```

```pdsl
UNIT GearPrdValidate

PURPOSE:
  Run deterministic validation on the GEAR-PRD.

DO:
  - RUN cfs toc <artifact-file>
  - RUN cfs validate --artifact <path> — must report PASS
  - RUN cfs validate-toc <artifact-file> — must report PASS
  - RUN cfs check-language <artifact-file> — must report PASS
  - RUN gts-validator over the artifact when it names GTS identifiers — must report no violations
```
