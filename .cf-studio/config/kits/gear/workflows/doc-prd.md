---
cf-studio: true
type: workflow
name: cf-gear-doc-prd
description: Invoke when the user asks to author, write, revise, or generate a gear PRD in gears-rust — e.g. "write the PRD for <gear>", "capture the gear's requirements", "spec the audit-log gear". Thin preset binding the GEAR-PRD artifact KIND, delegating authoring and review to the core cf-write-docs engine with gear kit resources. Requires a confirmed GEAR-INTENT.
version: 1.0
purpose: Thin preset that binds the GEAR-PRD artifact KIND and gear kit references, then delegates authoring and review to the core cf-write-docs workflow.
---

# cf-gear-doc-prd — Gear PRD authoring preset

Thin preset over the core `cf-write-docs` authoring engine: binds the
GEAR-PRD KIND and its kit resources, injects gear-specific rules (intent
carried into Gear Classification, requirements-only altitude), and delegates
the author → deterministic-gate → semantic-review loop. Authors no content
itself.

## Route

Preset: route, step count, and user gates are owned by the core cf-write-docs engine; this preset adds no questions of its own. Announce per the SKILL.md progress protocol.

## Prerequisite Checklist

- [ ] A confirmed GEAR-INTENT exists for this gear (`cfs list-ids`); run `cf-gear-intake` when missing

```pdsl
UNIT GearDocPrdPreset
PURPOSE: Bind the GEAR-PRD artifact KIND and gear kit references, then delegate authoring and review to the core cf-write-docs workflow.
STATE:
  SET ARTIFACT_KIND: GEAR-PRD (default GEAR-PRD, scope workflow_run)
DO:
  SET ARTIFACT_KIND = GEAR-PRD
  SET artifact_template = {gear_prd_template}
  SET artifact_rules = {gear_prd_rules}
  SET artifact_checklist = {gear_prd_checklist}
  SET artifact_example = {gear_prd_example}
  RUN resolve the gear's confirmed GEAR-INTENT and pass it as authoring input (classification carried verbatim into Gear Classification)
  LOAD {cf-studio-path}/.core/workflows/write-docs.md as the controlling authoring workflow
  CONTINUE WriteDocsBootstrap
RULES:
  ALWAYS bind ARTIFACT_KIND = GEAR-PRD and the four references (template, rules, checklist, example) before delegating to cf-write-docs
  ALWAYS inject {gear_prd_rules} as additional authoring rules into every author dispatch
  ALWAYS set the deterministic gate target to `cfs validate --artifact <path>` plus `gts-validator` over the artifact when it names GTS identifiers
  ALWAYS pass {gear_prd_checklist} to the semantic reviewer and {gear_prd_example} as the content-depth reference
  ALWAYS require a confirmed GEAR-INTENT before authoring; route to cf-gear-intake when none resolves
  ALWAYS carry ARTIFACT_KIND and the bound references as read-only preset data, never overriding cf-write-docs gates or verdicts
  NEVER author GEAR-PRD content in this preset; delegate all authoring and review to cf-write-docs
NOTES:
  cf-write-docs drives the author -> deterministic gate -> semantic review loop; this preset supplies the GEAR-PRD binding, the intent input, and the gear-specific gate extension.
```

## Next Steps

- `cf-gear-doc-design` once the PRD passes its gates
