---
cf-studio: true
type: workflow
name: cf-gear-lint
description: Invoke when the user asks about deterministic checks for a gear in gears-rust — e.g. "review the lint setup for <gear>", "why does dylint fail", "should this invariant become a lint", "configure Gears.toml/dylint.toml for the new gear". Reviews and configures the gear's architecture-lint posture (dylint rule set, skip/exclusion hygiene, clippy conformance) and drafts proposals for new deterministic rules where a review-only invariant repeats.
version: 1.0
purpose: Review and configure a gear's deterministic-check posture (dylint, clippy, GTS validation) and draft new-rule proposals for repeating invariants
---

# Gear Lint Workflow

ALWAYS open and follow `{cf-studio-path}/.core/skills/cf-studio/SKILL.md` FIRST WHEN {cf-studio-mode} is `off`

**Type**: Analysis / configuration
**Role**: Gear platform engineer
**Output**: clean `cargo gears lint --dylint` run for the gear; lint-config changes; optionally a new-rule proposal

---

## Routing

| User Intent | Route | Example |
|-------------|-------|---------|
| Review/fix a gear's lint posture | **lint.md** | "dylint fails on audit-log", `/cf-gear-lint audit-log` |
| Implement feature code | **implement.md** | "implement phase 2" |
| Prepare the PR | **pr.md** | "assemble the PR" |

---

## Overview

Architecture lints are the platform's deterministic memory: invariants
(contract purity, domain/infra separation, secure-ORM-only access, FIPS
hashers, GTS identifier hygiene, test file layout) live as dylint rules in
the external `cargo-gears-lints` crate, configured per repository by
`Gears.toml` (enable + skip list) and `dylint.toml` (rule parameters,
exclusions). This workflow reviews a gear against that posture, repairs
configuration hygiene, and — when a review keeps re-flagging the same
invariant — drafts a proposal for a new deterministic rule instead of
another checklist item.

The workflow changes configuration and drafts proposals; implementing new
lint rules happens in the `cargo-gears-lints` repository and is out of scope.

## Paths

- **Lint runner**: `cargo gears lint --dylint` (host repo)
- **Rule enablement**: `Gears.toml` (host repo; `[apps.<app>.dev.lint.dylint]` skip list — every skip carries an upstream issue link)
- **Rule parameters**: `dylint.toml` (host repo; `excluded_paths` is a migration list, not a dumping ground)
- **Clippy policy**: `clippy.toml` + workspace `[workspace.lints]` (host repo)
- **Defect-to-control map**: `docs/toolkit_unified_system/16_defect_class_to_control_map.md` (host repo)

## Route

5 steps → clean lint run (+ optional rule proposal). Questions: 1 decision (Step 4: apply fixes vs record exclusions vs draft a rule proposal). Announce the route before Step 1 and prefix every question per the SKILL.md progress protocol.

## Prerequisite Checklist

- [ ] The gear builds (`cargo build -p cf-gears-<name>`); lint noise on broken builds is meaningless
- [ ] `cargo gears` CLI is installed at the host repository's pinned version

---

## Steps

## Step 1: Run the lint baseline

// turbo
Run: `cargo gears lint --dylint` and capture the findings for the gear's
crates; also run `cargo clippy -p cf-gears-<name> -p cf-gears-<name>-sdk`.

## Step 2: Classify the findings

For each finding: (a) genuine violation — fix the code; (b) false positive
or migration debt — candidate for a parameterized exclusion; (c) missing
enforcement — an invariant the gear's review history keeps flagging that no
rule covers. Consult the defect-to-control map for the rule's intent before
classifying anything as (b).

## Step 3: Repair configuration hygiene

Verify: every skip in `Gears.toml` carries an upstream issue link; the gear
does not appear in `dylint.toml` `excluded_paths` unless it is genuinely
migration debt with an owner; no disallowed raw-ORM methods slipped past
`clippy.toml`. New gears start with zero skips and zero exclusions.

## Step 4: Decide the remediation

Present per finding class: fixes to apply now (option 1, recommended),
exclusions to record with issue links, and — for class (c) — a new-rule
proposal (rule intent, defect class, example violating/conforming code,
suggested rule ID range) to file against `cargo-gears-lints`
(**user gate — decision**).

## Step 5: Verify and close

// turbo
Run: `cargo gears lint --dylint` and the clippy commands again — must report
clean for the gear's crates. Fix and re-run until PASS. Present the
configuration diff and any drafted proposal (**user gate — confirmation**).

## Validation Criteria

- [ ] `cargo gears lint --dylint` clean for the gear's crates
- [ ] Clippy clean for both crates
- [ ] Every recorded skip/exclusion carries an issue link and owner
- [ ] Repeating review invariants have a drafted rule proposal, not a checklist entry

## Next Steps

- `cf-gear-pr` when the delivery is gate-clean
