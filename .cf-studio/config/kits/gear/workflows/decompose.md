---
cf-studio: true
type: workflow
name: cf-gear-decompose
description: Invoke when the user asks to plan a gear's implementation in gears-rust — e.g. "decompose the <gear> design", "break it into phases", "plan the implementation". Thin preset binding the GEAR-DECOMPOSITION artifact KIND, delegating authoring and review to the core cf-write-docs engine with gear kit resources. Enforces the phase contract: every phase ends buildable, runnable, and tested behind a named gate command.
version: 1.0
purpose: Thin preset that binds the GEAR-DECOMPOSITION artifact KIND and gear kit references, then delegates authoring and review to the core cf-write-docs workflow with the phase-gate check added to the deterministic gate.
---

# cf-gear-decompose — Gear decomposition preset

Thin preset over the core `cf-write-docs` engine: binds the
GEAR-DECOMPOSITION KIND, injects the phase contract (golden-path ordering,
verifiable completion criteria, a runnable gate per phase), and extends the
deterministic gate with the kit's phase-gate check. Authors no content itself.

## Route

Preset: route, step count, and user gates are owned by the core cf-write-docs engine, plus one plan-approval decision before the artifact is finalized. Announce per the SKILL.md progress protocol.

## Prerequisite Checklist

- [ ] GEAR-PRD and GEAR-DESIGN passed their deterministic gates; run their presets when missing

```pdsl
UNIT GearDecomposePreset
PURPOSE: Bind the GEAR-DECOMPOSITION artifact KIND and gear kit references, then delegate authoring and review to the core cf-write-docs workflow.
STATE:
  SET ARTIFACT_KIND: GEAR-DECOMPOSITION (default GEAR-DECOMPOSITION, scope workflow_run)
DO:
  SET ARTIFACT_KIND = GEAR-DECOMPOSITION
  SET artifact_template = {gear_decomposition_template}
  SET artifact_rules = {gear_decomposition_rules}
  SET artifact_checklist = {gear_decomposition_checklist}
  SET artifact_example = {gear_decomposition_example}
  RUN resolve the gear's gated GEAR-DESIGN and GEAR-PRD as authoring inputs; order phases by the golden path (SDK contract, gear host and registration, seams and features)
  LOAD {cf-studio-path}/.core/workflows/write-docs.md as the controlling authoring workflow
  CONTINUE WriteDocsBootstrap
RULES:
  ALWAYS bind ARTIFACT_KIND = GEAR-DECOMPOSITION and the four references before delegating to cf-write-docs
  ALWAYS inject {gear_decomposition_rules} as additional authoring rules into every author dispatch
  ALWAYS set the deterministic gate target to `cfs validate --artifact <path>` plus `python3 {scripts}/check_phase_gates.py <path>` — every phase must carry a runnable gate command
  ALWAYS pass {gear_decomposition_checklist} to the semantic reviewer and {gear_decomposition_example} as the content-depth reference
  ALWAYS require the contributor's approval of the phase plan before finalizing the artifact (user gate — decision)
  ALWAYS require the final phase's gate to include the full local gate and the end-to-end run
  ALWAYS carry ARTIFACT_KIND and the bound references as read-only preset data, never overriding cf-write-docs gates or verdicts
  NEVER author GEAR-DECOMPOSITION content in this preset; delegate all authoring and review to cf-write-docs
NOTES:
  The approved decomposition is the implementation contract: the implementation workflow executes it phase by phase and never advances past a failing gate; material plan changes reroute through this preset.
```

## Next Steps

- Scaffold and implementation routes (next kit release) execute the approved phases
