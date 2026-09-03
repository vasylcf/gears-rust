---
cf-studio: true
type: workflow
name: cf-gear-doc-adr
description: Invoke when the user asks to record a gear architecture decision in gears-rust — e.g. "record the decision on <topic>", "write an ADR for <gear>", "document why we chose X over Y". Thin preset binding the GEAR-ADR artifact KIND, delegating authoring and review to the core cf-write-docs engine with gear kit resources. The resulting ADR must be referenced from the gear's GEAR-DESIGN.
version: 1.0
purpose: Thin preset that binds the GEAR-ADR artifact KIND and gear kit references, then delegates authoring and review to the core cf-write-docs workflow.
---

# cf-gear-doc-adr — Gear ADR authoring preset

Thin preset over the core `cf-write-docs` engine: binds the GEAR-ADR KIND,
injects gear-specific rules (real dilemma with at least two options, concrete
confirmation mechanism, the record lives in the gear it governs), and
delegates the authoring loop. Authors no content itself.

## Route

Preset: route, step count, and user gates are owned by the core cf-write-docs engine; this preset adds no questions of its own. Announce per the SKILL.md progress protocol.

## Prerequisite Checklist

- [ ] The gear's GEAR-DESIGN exists (the ADR ID must be added to its Architecture Drivers in the same change)

```pdsl
UNIT GearDocAdrPreset
PURPOSE: Bind the GEAR-ADR artifact KIND and gear kit references, then delegate authoring and review to the core cf-write-docs workflow.
STATE:
  SET ARTIFACT_KIND: GEAR-ADR (default GEAR-ADR, scope workflow_run)
DO:
  SET ARTIFACT_KIND = GEAR-ADR
  SET artifact_template = {gear_adr_template}
  SET artifact_rules = {gear_adr_rules}
  SET artifact_checklist = {gear_adr_checklist}
  SET artifact_example = {gear_adr_example}
  SET the artifact filename = NNNN-{cpt-id}.md in the gear's docs/ADR/ directory
  LOAD {cf-studio-path}/.core/workflows/write-docs.md as the controlling authoring workflow
  CONTINUE WriteDocsBootstrap
RULES:
  ALWAYS bind ARTIFACT_KIND = GEAR-ADR and the four references before delegating to cf-write-docs
  ALWAYS inject {gear_adr_rules} as additional authoring rules into every author dispatch
  ALWAYS set the deterministic gate target to `cfs validate --artifact <path>` (which enforces the GEAR-DESIGN reference coverage)
  ALWAYS pass {gear_adr_checklist} to the semantic reviewer and {gear_adr_example} as the content-depth reference
  ALWAYS add the new ADR ID to the gear DESIGN's Architecture Drivers in the same change set
  ALWAYS carry ARTIFACT_KIND and the bound references as read-only preset data, never overriding cf-write-docs gates or verdicts
  NEVER author GEAR-ADR content in this preset; delegate all authoring and review to cf-write-docs
NOTES:
  Contested decisions attach evidence (a SPIKE with measurements, or a dated alternatives evaluation); superseded evidence stays in the tree marked as dated evidence.
```

## Next Steps

- Return to `cf-gear-doc-design` to reflect the decision, or `cf-gear-decompose` when the design is complete
