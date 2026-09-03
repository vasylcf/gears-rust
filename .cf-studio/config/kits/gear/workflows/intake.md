---
cf-studio: true
type: workflow
name: cf-gear-intake
description: Invoke when the user wants to start any gear work in gears-rust — "create a new gear", "add a plugin to <gear>", "spec a gear", "fix <gear>", or an unclassified gear-shaped request. Classifies the request (new gear | plugin | docs-only spec | small fix), runs the mandatory similar-gear lookup over the shipped catalog, recommends the archetype and reuse profile, and records the confirmed route as a GEAR-INTENT artifact — the root of the gear's traceability chain.
version: 1.0
purpose: Classify a gear-creation request, run similar-gear lookup, record the reuse profile and confirmed route as GEAR-INTENT
---

# Gear Intake Workflow

ALWAYS open and follow `{cf-studio-path}/.core/skills/cf-studio/SKILL.md` FIRST WHEN {cf-studio-mode} is `off`

**Type**: Intake
**Role**: Gear platform guide
**Output**: `{artifacts_dir}/GEAR-INTENT/{yyyy-mm-dd}-{slug}.md`

---

## Routing

| User Intent | Route | Example |
|-------------|-------|---------|
| Start any gear work (unclassified) | **intake.md** | "we need an audit trail gear", `/cf-gear-intake` |
| Author a specific document with intent confirmed | **doc-prd.md** / **doc-design.md** / **doc-adr.md** | "write the PRD for audit-log" |
| Plan implementation phases | **decompose.md** | "decompose the audit-log design" |

---

## Overview

Produces one GEAR-INTENT artifact per request: the classification (new gear,
plugin, docs-only spec, or small fix), the verbatim similar-gear lookup
results, the archetype and reuse profile (reference gear, core toolkit
libraries, platform dependencies), the document set, and the confirmed route.
Every later artifact for this gear cites the intent that commissioned it.

The workflow authors the artifact per `{gear_intent_rules}` and validates it
against `{gear_intent_checklist}`. Classification discipline: a request
touching public contracts is never a small fix; a capability an existing gear
should own routes to a plugin or feature on that gear.

## Paths

- **Template**: `{gear_intent_template}`
- **Rules**: `{gear_intent_rules}`
- **Checklist**: `{gear_intent_checklist}`
- **Example**: `{gear_intent_example}`
- **Catalog**: `{gear_catalog}`
- **Lookup script**: `{scripts}/find_similar_gears.py`
- **Artifacts registry**: `{cf-studio-path}/config/artifacts.toml`

## Route

6 steps → GEAR-INTENT. Questions: 2 decisions (Step 3: reuse-or-build; Step 6: route confirmation), plus clarifications in Step 1 only when the request is ambiguous. Steps 2, 4, 5 run without questions. Announce the route before Step 1 and prefix every question per the SKILL.md progress protocol.

## Prerequisite Checklist

- [ ] The gear catalog resource is installed (`{gear_catalog}` exists); note its snapshot date
- [ ] `{cf-studio-path}/config/artifacts.toml` is writable for registration

---

## Steps

## Step 1: Capture the request

Record the need in the requester's terms: problem or capability, who needs
it, acceptance outcome, originating issue/task if any. Ask only when the
subject or outcome is genuinely ambiguous (**user gate — decision**, only
when needed).

## Step 2: Similar-gear lookup

// turbo
Run: `python3 {scripts}/find_similar_gears.py --catalog {gear_catalog} --capabilities <caps> --keywords <domain keywords>`
Record the output verbatim — an empty result must show the query used. Never
skip or truncate this step.

## Step 3: Classify and decide reuse-or-build

Apply the routing discipline: public-contract impact ⇒ never small fix;
capability owned by an existing gear ⇒ plugin or feature on that gear.
Present the classification, the closest matches with verdicts, and the
reuse-or-build call with the losing options named. Option 1 is the
recommendation (**user gate — decision**).
STOP WHEN the user rejects all options — capture what is missing and re-run from Step 1.

## Step 4: Author the GEAR-INTENT artifact

Load `{gear_intent_template}` and `{gear_intent_example}`. Follow
`{gear_intent_rules}`: fill Request, Classification, Similar Gears (verbatim),
Reuse Profile (archetype, reference gear, core/optional libraries, platform
deps — from the lookup's archetype suggestion), Document Set per the
classification, and Route Decision. Set the ID
`cpt-{system}-intent-{slug}`; verify uniqueness with `cfs list-ids`;
register the file in `artifacts.toml`.

## Step 5: Deterministic gate

// turbo
Run: `cfs toc <artifact-file>` then `cfs validate --artifact <artifact-file>`,
`cfs validate-toc <artifact-file>`, and `cfs check-language <artifact-file>`.
Fix and re-run until all report PASS.

## Step 6: Confirm the route

Load `{gear_intent_checklist}` and self-review (INT-001…INT-007). Present the
intent summary and the route: the workflow sequence and the first next step
(new gear ⇒ `cf-gear-doc-prd`; plugin with existing host seam ⇒ scaffold
route; small fix ⇒ implementation with scope in the issue/PR). Wait for the
requester's confirmation and record it in Route Decision
(**user gate — decision**).

## Validation Criteria

- [ ] GEAR-INTENT registered and passing `cfs validate --artifact`
- [ ] Similar-gear lookup output recorded verbatim with the catalog snapshot date
- [ ] Classification consistent with the document-set table
- [ ] Route confirmed by the requester with a date

## Next Steps

- New gear or docs-only: `cf-gear-doc-prd`
- Plugin with host specs in place: scaffold route (next kit release)
- Small fix: implementation route with scope recorded in the issue/PR
