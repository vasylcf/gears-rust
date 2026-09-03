---
name: cf
description: "Invoke when the user asks to create a new Gear for gears-rust, add a plugin to an existing gear, write gear specification documents (intent, PRD, design, ADR, decomposition, feature), decompose a gear design into buildable phases, scaffold gear crates, run phased gear implementation with gates, or prepare a gear pull request. Kit `gear` extensions — artifacts: GEAR-INTENT, GEAR-PRD, GEAR-DESIGN, GEAR-ADR, GEAR-DECOMPOSITION, GEAR-FEATURE, GEAR-IMPL-REPORT; data: gear catalog for similar-gear lookup; scripts: find_similar_gears, check_gear_layout, check_phase_gates."
---

# Constructor Studio Skill — Kit `gear`

Kit `gear` extensions: the guided, gate-checked path for creating Gears in gears-rust — from intent to a review-ready pull request.

## Pipeline

```
gear-creation ask
  → intake → GEAR-INTENT (classification, similar-gear lookup, reuse profile, route)
new gear / docs-only routes
  → doc-prd → GEAR-PRD (WHAT; gear classification carried from intent)
  → doc-design → GEAR-DESIGN (structure, GTS types, components, seams; FR/NFR coverage)
  → doc-adr → GEAR-ADR (per decision; referenced from DESIGN)
  → decompose → GEAR-DECOMPOSITION (phases: scope + criteria + gate command)
implementation routes
  → scaffold → crates + registration surface (gate: build, lints, layout, names)
  → implement → per-phase execution (gate: gear-scoped CI) → GEAR-IMPL-REPORT per phase
  → pr → PR package (conventional commits, body digest, review ledger, size check)
```

The GEAR-INTENT is the root of the traceability chain: every later artifact
cites the intent that commissioned it, and the PRD's requirement IDs thread
through design, decomposition, and reports.

## Artifact kinds

| Kind | Semantic intent | Resources |
|------|-----------------|-----------|
| GEAR-INTENT | One classified gear-creation request: class, similar gears, reuse profile, confirmed route. | `{gear_intent_rules}`, `{gear_intent_template}`, `{gear_intent_checklist}`, `{gear_intent_example}` |
| GEAR-PRD | Requirements-only WHAT of one gear, with classification from intake. | `{gear_prd_rules}`, `{gear_prd_template}`, `{gear_prd_checklist}`, `{gear_prd_example}` |
| GEAR-DESIGN | HOW the requirements are realized: crates, GTS types, components, seams, storage. | `{gear_design_rules}`, `{gear_design_template}`, `{gear_design_checklist}`, `{gear_design_example}` |
| GEAR-ADR | One gear architecture decision with real options and a confirmable outcome. | `{gear_adr_rules}`, `{gear_adr_template}`, `{gear_adr_checklist}`, `{gear_adr_example}` |
| GEAR-DECOMPOSITION | Ordered phases, each ending buildable/runnable/tested behind a named gate command. | `{gear_decomposition_rules}`, `{gear_decomposition_template}`, `{gear_decomposition_checklist}`, `{gear_decomposition_example}` |
| GEAR-FEATURE | Optional behavior contract for complex flows/states, with testable DoD. | `{gear_feature_rules}`, `{gear_feature_template}`, `{gear_feature_checklist}`, `{gear_feature_example}` |
| GEAR-IMPL-REPORT | Gate-audited close-out of one implementation phase. | `{gear_impl_report_rules}`, `{gear_impl_report_template}`, `{gear_impl_report_checklist}`, `{gear_impl_report_example}` |

## Deterministic gates

Stage exits reuse the host repository's own commands (never kit-invented
checks): `cfs validate --artifact` + `gts-validator` for documents;
`cargo build/clippy -p`, `cargo gears lint --dylint`, `make validate-gear-names`
plus `{scripts}/check_gear_layout.py` for scaffolds; `make gear-ci GEAR=<name>`
per implementation phase; `make check` + e2e at completion. Express modes may
auto-proceed confirmation gates but never relax a deterministic gate.

## Data

`{gear_catalog}` — the machine-readable gear dependency graph (nodes,
typed edges, docs/GTS attributes). Queried at intake via
`python3 {scripts}/find_similar_gears.py --catalog {gear_catalog} ...`.
Regenerate from a gears-rust checkout with the kit repository's
`tools/build_gear_catalog.py`.

## Workflows

| Skill | Type | Workflow | Output |
|-------|------|----------|--------|
| `cf-gear-intake` | intake | `{workflow_intake}` | GEAR-INTENT artifact (classification, lookup, reuse profile, route) |
| `cf-gear-doc-prd` | authoring preset | `{workflow_doc_prd}` | GEAR-PRD artifact via cf-write-docs |
| `cf-gear-doc-design` | authoring preset | `{workflow_doc_design}` | GEAR-DESIGN artifact via cf-write-docs |
| `cf-gear-doc-adr` | authoring preset | `{workflow_doc_adr}` | GEAR-ADR artifact via cf-write-docs |
| `cf-gear-doc-feature` | authoring preset | `{workflow_doc_feature}` | GEAR-FEATURE artifact via cf-write-docs (optional kind) |
| `cf-gear-decompose` | planning preset | `{workflow_decompose}` | GEAR-DECOMPOSITION artifact + phase-gate check |
| `cf-gear-scaffold` | scaffolding | `{workflow_scaffold}` | SDK + gear crates, registration surface, passing scaffold gate |
| `cf-gear-implement` | implementation preset | `{workflow_implement}` | phase-by-phase code via cf-coding + GEAR-IMPL-REPORT per phase |
| `cf-gear-lint` | analysis | `{workflow_lint}` | clean dylint/clippy posture; lint-config changes; new-rule proposals |
| `cf-gear-pr` | delivery | `{workflow_pr}` | commits + structured PR body + review ledger + size check |

## Progress protocol

Every workflow run follows one interaction contract, so the user always
knows where they are and what remains:

- Announce the route before the first step: step count and the number of
  user questions, from the workflow's `## Route` block.
- Prefix every question with `[cf-gear-<name> · step k/N · questions left: M]`.
- Gates are typed: `**user gate — decision**` (the user chooses between
  options; option 1 is always the recommendation) or
  `**user gate — confirmation**` (proceed/adjust).
- Recovery never asks: when a deterministic gate fails, fix and re-run it;
  questions are for decisions, not for permission to repair.
- Deterministic gates are never relaxed in any mode; an express mode may
  auto-proceed confirmation gates only, logging each auto-confirmation.
