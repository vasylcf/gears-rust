---
cf-studio: true
type: workflow
name: cf-gear-doc-design
description: Invoke when the user asks to author, write, revise, or generate a gear technical design in gears-rust — e.g. "design the <gear> gear", "allocate the requirements to components", "define the gear's crates, GTS types, seams, storage". Thin preset binding the GEAR-DESIGN artifact KIND, delegating authoring and review to the core cf-write-docs engine with gear kit resources. Requires a gated GEAR-PRD.
version: 1.0
purpose: Thin preset that binds the GEAR-DESIGN artifact KIND and gear kit references, then delegates authoring and review to the core cf-write-docs workflow.
---

# cf-gear-doc-design — Gear design authoring preset

Thin preset over the core `cf-write-docs` engine: binds the GEAR-DESIGN KIND,
injects gear-specific rules (full FR/NFR driver coverage, canonical layout
read first, registration surface declared, storage stated even when
stateless), and delegates the authoring loop. Authors no content itself.

## Route

Preset: route, step count, and user gates are owned by the core cf-write-docs engine; this preset adds no questions of its own. Announce per the SKILL.md progress protocol.

## Prerequisite Checklist

- [ ] The gear's GEAR-PRD passed its deterministic gate; run `cf-gear-doc-prd` when missing
- [ ] The canonical layout reference is readable in the host repository (docs/toolkit_unified_system/02_gear_layout_and_sdk_pattern.md)

```pdsl
UNIT GearDocDesignPreset
PURPOSE: Bind the GEAR-DESIGN artifact KIND and gear kit references, then delegate authoring and review to the core cf-write-docs workflow.
STATE:
  SET ARTIFACT_KIND: GEAR-DESIGN (default GEAR-DESIGN, scope workflow_run)
DO:
  SET ARTIFACT_KIND = GEAR-DESIGN
  SET artifact_template = {gear_design_template}
  SET artifact_rules = {gear_design_rules}
  SET artifact_checklist = {gear_design_checklist}
  SET artifact_example = {gear_design_example}
  RUN resolve the gear's gated GEAR-PRD and pass it as authoring input (every fr and nfr ID must land in the driver tables)
  LOAD {cf-studio-path}/.core/workflows/write-docs.md as the controlling authoring workflow
  CONTINUE WriteDocsBootstrap
RULES:
  ALWAYS bind ARTIFACT_KIND = GEAR-DESIGN and the four references before delegating to cf-write-docs
  ALWAYS inject {gear_design_rules} as additional authoring rules into every author dispatch
  ALWAYS set the deterministic gate target to `cfs validate --artifact <path>` (which enforces PRD fr/nfr coverage) plus `gts-validator` over the artifact when it names GTS identifiers
  ALWAYS pass {gear_design_checklist} to the semantic reviewer and {gear_design_example} as the content-depth reference
  ALWAYS require a gated GEAR-PRD before authoring; route to cf-gear-doc-prd when none resolves
  ALWAYS carry ARTIFACT_KIND and the bound references as read-only preset data, never overriding cf-write-docs gates or verdicts
  NEVER author GEAR-DESIGN content in this preset; delegate all authoring and review to cf-write-docs
NOTES:
  Decision dilemmas discovered during design authoring route to cf-gear-doc-adr; the resulting ADR IDs return into this design's Architecture Drivers.
```

## Next Steps

- `cf-gear-doc-adr` for each decision dilemma surfaced by the design
- `cf-gear-decompose` once the design passes its gates
