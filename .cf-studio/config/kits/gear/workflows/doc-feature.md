---
cf-studio: true
type: workflow
name: cf-gear-doc-feature
description: Invoke when the user asks to spec a complex gear behavior in gears-rust — e.g. "spec the retention sweep", "write the FEATURE for <behavior>", "define the flows/states and definition of done". Thin preset binding the GEAR-FEATURE artifact KIND, delegating authoring and review to the core cf-write-docs engine with gear kit resources. FEATURE is optional — most behavior belongs in GEAR-DESIGN.
version: 1.0
purpose: Thin preset that binds the GEAR-FEATURE artifact KIND and gear kit references, then delegates authoring and review to the core cf-write-docs workflow.
---

# cf-gear-doc-feature — Gear feature authoring preset

Thin preset over the core `cf-write-docs` engine: binds the GEAR-FEATURE
KIND, injects gear-specific rules (optional artifact — existence check first;
failure paths are part of behavior; every DoD item testable), and delegates
the authoring loop. Authors no content itself.

## Route

Preset: route, step count, and user gates are owned by the core cf-write-docs engine; this preset adds one existence decision (is this behavior complex enough for its own contract?) before delegating. Announce per the SKILL.md progress protocol.

## Prerequisite Checklist

- [ ] The implementing fr IDs exist in the gear's GEAR-PRD
- [ ] The delivering phase exists in the gear's GEAR-DECOMPOSITION (or is being planned)

```pdsl
UNIT GearDocFeaturePreset
PURPOSE: Bind the GEAR-FEATURE artifact KIND and gear kit references, then delegate authoring and review to the core cf-write-docs workflow.
STATE:
  SET ARTIFACT_KIND: GEAR-FEATURE (default GEAR-FEATURE, scope workflow_run)
DO:
  RUN confirm the behavior warrants its own contract (flows, states, or algorithms beyond GEAR-DESIGN prose); route the content into GEAR-DESIGN when it does not (user gate — decision)
  SET ARTIFACT_KIND = GEAR-FEATURE
  SET artifact_template = {gear_feature_template}
  SET artifact_rules = {gear_feature_rules}
  SET artifact_checklist = {gear_feature_checklist}
  SET artifact_example = {gear_feature_example}
  SET the artifact filename = NNNN-{cpt-id}.md in the gear's docs/features/ directory
  LOAD {cf-studio-path}/.core/workflows/write-docs.md as the controlling authoring workflow
  CONTINUE WriteDocsBootstrap
RULES:
  ALWAYS bind ARTIFACT_KIND = GEAR-FEATURE and the four references before delegating to cf-write-docs
  ALWAYS inject {gear_feature_rules} as additional authoring rules into every author dispatch
  ALWAYS set the deterministic gate target to `cfs validate --artifact <path>`
  ALWAYS pass {gear_feature_checklist} to the semantic reviewer and {gear_feature_example} as the content-depth reference
  ALWAYS carry ARTIFACT_KIND and the bound references as read-only preset data, never overriding cf-write-docs gates or verdicts
  NEVER author GEAR-FEATURE content in this preset; delegate all authoring and review to cf-write-docs
NOTES:
  FEATURE is the optional artifact of the set: gears ship far fewer FEATUREs than ADRs in practice, and implementation without FEATUREs runs directly from DESIGN plus DECOMPOSITION.
```

## Next Steps

- `cf-gear-decompose` (map the feature to its delivering phase), or implementation once phases exist
