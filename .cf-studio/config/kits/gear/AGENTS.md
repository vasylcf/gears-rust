# Constructor Studio Kit: Gear Creation (`gear`)

Agent quick reference.

## What it is

The guided, gate-checked path for creating Gears in gears-rust: a request is
classified at intake (GEAR-INTENT, with similar-gear lookup over the shipped
catalog), specified through gear document kinds (PRD → DESIGN → ADRs →
DECOMPOSITION, FEATURE optional), then scaffolded and implemented phase by
phase — every stage exiting through the host repository's own deterministic
gate commands, with a GEAR-IMPL-REPORT recording each phase close.

## Artifact kinds

| Kind | When to use | References |
|------|-------------|------------|
| GEAR-INTENT | First, for every gear-creation request; the traceability root. | `{gear_intent_rules}`, `{gear_intent_template}`, `{gear_intent_checklist}`, `{gear_intent_example}` |
| GEAR-PRD | New gear or material requirement change; requirements-only. | `{gear_prd_rules}`, `{gear_prd_template}`, `{gear_prd_checklist}`, `{gear_prd_example}` |
| GEAR-DESIGN | Allocating requirements to crates, components, GTS types, seams, storage. | `{gear_design_rules}`, `{gear_design_template}`, `{gear_design_checklist}`, `{gear_design_example}` |
| GEAR-ADR | One real decision dilemma; filename NNNN-{cpt-id}.md in the gear's docs/ADR/. | `{gear_adr_rules}`, `{gear_adr_template}`, `{gear_adr_checklist}`, `{gear_adr_example}` |
| GEAR-DECOMPOSITION | Before implementation; phases with scope, criteria, gate command. | `{gear_decomposition_rules}`, `{gear_decomposition_template}`, `{gear_decomposition_checklist}`, `{gear_decomposition_example}` |
| GEAR-FEATURE | Only for behavior too complex for DESIGN prose. | `{gear_feature_rules}`, `{gear_feature_template}`, `{gear_feature_checklist}`, `{gear_feature_example}` |
| GEAR-IMPL-REPORT | At every phase close, after all gates pass. | `{gear_impl_report_rules}`, `{gear_impl_report_template}`, `{gear_impl_report_checklist}`, `{gear_impl_report_example}` |

## Workflows

| Skill | Purpose |
| --- | --- |
| `cf-gear-intake` | Classify a gear request, run the mandatory similar-gear lookup, record the reuse profile and confirmed route (GEAR-INTENT). |
| `cf-gear-doc-prd` | Author the gear PRD from the confirmed intent (requirements-only altitude). |
| `cf-gear-doc-design` | Author the gear design; the gate enforces full PRD fr/nfr coverage. |
| `cf-gear-doc-adr` | Record one gear decision; the gate enforces the DESIGN back-reference. |
| `cf-gear-doc-feature` | Spec one complex behavior (optional kind; existence check first). |
| `cf-gear-decompose` | Plan implementation phases; the gate runs check_phase_gates.py — every phase carries a runnable gate command. |
| `cf-gear-scaffold` | Create the crates and full registration surface in the canonical order; exit through the scaffold gate. |
| `cf-gear-implement` | Execute approved phases via cf-coding; per-phase gate + GEAR-IMPL-REPORT; never advances past a failing gate. |
| `cf-gear-lint` | Review/repair the gear's dylint/clippy posture; draft new-rule proposals for repeating invariants. |
| `cf-gear-pr` | Assemble the review-ready PR: conventional signed-off commits, body digest from reports, review ledger, size check. |

## Rules

- The intake classification routes ceremony: new gear ⇒ full document set;
  plugin ⇒ reuse host specs; small fix ⇒ no specs, scope in the issue/PR —
  but public-contract changes are never small fixes.
- Deterministic gates are the host repository's own commands; a stage is not
  complete while its gate fails, and no mode of operation relaxes a gate.
- Every artifact kind's rules file ends in a Validate unit — run it before
  reporting the artifact done.
- The similar-gears lookup at intake is mandatory; record its output verbatim.
- ADRs live in the gear they govern and must be referenced from that gear's
  DESIGN Architecture Drivers.
