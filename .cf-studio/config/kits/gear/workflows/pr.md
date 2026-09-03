---
cf-studio: true
type: workflow
name: cf-gear-pr
description: Invoke when the user asks to prepare or update a gear pull request in gears-rust — e.g. "assemble the PR for audit-log", "prepare the pull request", "update the PR after review". Prepares the review-ready package: conventional commits with DCO sign-off, a structured PR body digest built from the GEAR-IMPL-REPORTs, a per-reviewer response ledger on revision rounds, and a size check against the reviewability ceiling.
version: 1.0
purpose: Assemble the review-ready gear PR package — commits, structured body digest, review ledger, size check
---

# Gear PR Workflow

ALWAYS open and follow `{cf-studio-path}/.core/skills/cf-studio/SKILL.md` FIRST WHEN {cf-studio-mode} is `off`

**Type**: Delivery
**Role**: Gear contributor
**Output**: commits + PR body (+ review-ledger update on revision rounds)

---

## Routing

| User Intent | Route | Example |
|-------------|-------|---------|
| Prepare or update the gear PR | **pr.md** | "assemble the PR", `/cf-gear-pr` |
| Close remaining phases first | **implement.md** | "finish phase 4" |
| Fix lint posture first | **lint.md** | "dylint is red" |

---

## Overview

Assembles the delivery the way exemplary gear PRs ship: atomic conventional
commits with DCO sign-off, a PR body that is a structured spec digest (what
is included per directory with counts, the ADR decision table, dependency
status with merge state, what is deliberately not here, open questions with
blocking markers, validation evidence from the GEAR-IMPL-REPORTs), and — on
revision rounds — a per-reviewer response ledger. A size check warns beyond
the reviewability ceiling and recommends the staged golden-path sequence
(PRD → DESIGN+ADRs → SDK → gear host) instead of one oversized PR.

The workflow prepares and audits; it never pushes or opens the PR without
the contributor's explicit go.

## Paths

- **Reports**: the gear's GEAR-IMPL-REPORT artifacts (validation evidence)
- **Commit convention**: `<type>(<gear>): <description>` (15 accepted types, host CONTRIBUTING §2.8); DCO `Signed-off-by` on every commit
- **PR checklist**: host CONTRIBUTING §2.9 (Description / Type of Change / Testing / Documentation / Checklist / Related Issues)
- **Size ceiling**: ~8.5k changed lines / ~70 files — beyond it, human line-level review demonstrably stops

## Route

6 steps → PR package. Questions: 2 (Step 4: body approval — decision; Step 6: push/open go — decision). Announce the route before Step 1 and prefix every question per the SKILL.md progress protocol.

## Prerequisite Checklist

- [ ] Every decomposition phase in scope has a passing GEAR-IMPL-REPORT (or the delivery is an approved docs-only/small-fix scope)
- [ ] The working tree passes the gates the reports claim (`make gear-ci GEAR=<name>` re-run is cheap — trust but verify)

---

## Steps

## Step 1: Audit the delivery state

// turbo
Run: `git status`, `git log --oneline <base>..HEAD`, `git diff --stat <base>...HEAD`,
and re-run the gear-scoped gate: `make gear-ci GEAR=<name>`.
Then the pre-push workspace set the gear gate does not cover:
- `cargo check --workspace --all-targets` — cross-gear compile breakage
- `make clippy` (workspace, includes the `cargo hack` per-feature matrix —
  the only place feature combinations like `pgq` without `integration` are
  checked; per-crate clippy never sees them)
- `make deny` whenever the delivery touched any `Cargo.toml` (new members,
  optional deps, dev-deps — the dependency graph changed)
- for a `db` gear: the Docker DB lane `make test-<gear>-db`
Collect the GEAR-IMPL-REPORTs. STOP WHEN a claimed-passing gate fails —
route back to `cf-gear-implement`.

## Step 2: Size check

Compare the diff against the ceiling (~8.5k lines / ~70 files). Beyond it,
recommend splitting along the golden path (docs PR → SDK PR → gear-host PR)
and stop at the split proposal. Within it, continue.

## Step 3: Shape the commits

Atomic conventional commits `<type>(<gear>): <description>`; every commit
carries DCO `Signed-off-by` (`git commit -s`). Registration surface changes
land in the same commit as the crates they register. Public-contract changes
carry a version-bump justification in the commit or PR body.

## Step 4: Assemble the PR body

Build the digest from the artifacts and reports:
- inventory per directory with counts (files, requirements, components)
- ADR decision table (ADR / decision / status)
- dependency status: linked PRs/issues with merge state
- "What is deliberately not here" — named owners for excluded behavior
- open questions marked blocking / non-blocking, with what keeps deferral cheap
- validation evidence: gate commands and outcomes from the GEAR-IMPL-REPORTs
  (e.g. "cfs validate: N artifacts, 0 errors"; "make gear-ci: pass")
- the host PR checklist filled (Description / Type of Change / Testing /
  Documentation / Checklist / Related Issues)
Present for approval (**user gate — decision**).

## Step 5: Revision-round ledger (when updating an existing PR)

For each reviewer with open comments, append a ledger entry to the PR body:
"From @<reviewer> — <n> comments: <k> addressed (list), <m> open (why)".
Answer review threads in-thread; the ledger summarizes, it never replaces
thread answers. Self-annotate non-obvious diff hunks with inline comments.

## Step 6: Deliver

Present the final package: commit list, body, ledger. On explicit go, push
and open/update the PR (**user gate — decision**). Record the PR reference
in the final GEAR-IMPL-REPORT's Next Phase section.

## Validation Criteria

- [ ] Gear-scoped gate re-verified at delivery tip
- [ ] Pre-push workspace set clean: workspace check + `make clippy` (with the feature matrix), `make deny` on Cargo.toml changes, the Docker DB lane for db gears
- [ ] Every commit conventional and signed off
- [ ] Body carries inventory, ADR table, exclusions, open questions, validation evidence
- [ ] Size within the ceiling, or a split proposal delivered instead
- [ ] Nothing pushed without the explicit go

## Next Steps

- Reviewer feedback rounds re-enter at Step 5
- Post-merge: retention of reports alongside the gear's docs is the host project's call
