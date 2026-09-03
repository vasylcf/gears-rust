---
cf-studio: true
type: workflow
name: cf-gear-scaffold
description: Invoke when the user asks to scaffold a new gear or plugin in gears-rust — e.g. "scaffold the <gear> gear", "create the crates for <gear>", "set up the gear skeleton and registration". Creates the SDK and gear crates in the canonical order, completes the full registration surface (workspace, example server, feature flag, cargo-shear exemptions, e2e config), and exits through the scaffold gate (build, clippy, dylint, layout and name checks). Requires a confirmed GEAR-INTENT; for a new gear, an approved GEAR-DESIGN.
version: 1.0
purpose: Scaffold gear crates and the full registration surface in the canonical order, exiting through the deterministic scaffold gate
---

# Gear Scaffold Workflow

ALWAYS open and follow `{cf-studio-path}/.core/skills/cf-studio/SKILL.md` FIRST WHEN {cf-studio-mode} is `off`

**Type**: Scaffolding
**Role**: Gear platform engineer
**Output**: SDK + gear crates, registration surface, passing scaffold gate

---

## Routing

| User Intent | Route | Example |
|-------------|-------|---------|
| Create the crates and registration for a gear/plugin | **scaffold.md** | "scaffold audit-log", `/cf-gear-scaffold audit-log` |
| Plan phases first | **decompose.md** | "decompose the design" |
| Implement approved phases | **implement.md** | "implement phase 1" |

---

## Overview

Creates a new gear's (or plugin's) crates per the canonical layout and wires
the complete registration surface, in the fixed order the platform documents.
The workflow generates structure and wiring, not feature logic — feature code
belongs to `cf-gear-implement`. Everything scaffolded must leave the
workspace buildable: the exit is the deterministic scaffold gate.

Source contracts: the confirmed GEAR-INTENT (name, capabilities, deps,
archetype, reference gear) and, for a new gear, the approved GEAR-DESIGN
(crates, components, seams, storage). The canonical layout reference and the
"Adding a New Gear" checklist in the host repository are read before any file
is created.

## Paths

- **Intent**: the gear's GEAR-INTENT artifact (`cfs list-ids`)
- **Design**: the gear's GEAR-DESIGN artifact (new gears)
- **Layout reference**: `docs/toolkit_unified_system/02_gear_layout_and_sdk_pattern.md` (host repo)
- **Checklist reference**: `docs/toolkit_unified_system/10_checklists_and_templates.md` (host repo)
- **Canonical example**: `examples/toolkit/users-info/` (host repo)
- **Layout check**: `{scripts}/check_gear_layout.py`

## Route

7 steps → scaffolded crates + registration + passing gate. Questions: 1 decision (Step 2: scaffold plan approval). Steps 3–7 run without questions. Announce the route before Step 1 and prefix every question per the SKILL.md progress protocol.

## Prerequisite Checklist

- [ ] A confirmed GEAR-INTENT resolves for this gear; run `cf-gear-intake` when missing
- [ ] For a new gear: GEAR-DESIGN passed its gate; run `cf-gear-doc-design` when missing
- [ ] The gears-rust working copy builds clean before scaffolding (`cargo build` at the tip)

---

## Steps

## Step 1: Resolve contracts and read the references

Resolve the GEAR-INTENT (and GEAR-DESIGN for a new gear). Read the layout
reference and the "Adding a New Gear" checklist in the host repository; study
the reference gear named by the intent's reuse profile (when unsure about a
pattern, read the corresponding file in the canonical example first).
STOP WHEN no confirmed GEAR-INTENT resolves.

## Step 2: Present the scaffold plan

Present: crate names (`cf-gears-<name>` / `cf-gears-<name>-sdk`, lib names
`<name>` / `<name>_sdk`), the gear macro declaration (name, capabilities,
deps), the dependency blocks from the archetype profile, the file-creation
order, and the registration surface to be touched. Option 1 is the
recommendation (**user gate — decision**).

## Step 3: Create the SDK crate

Fixed order, SDK first — consumers compile against the seam before the gear
exists: `Cargo.toml` (workspace-inherited fields, `[lints] workspace = true`),
`src/lib.rs`, `src/api.rs` (traits), `src/models.rs`, `src/errors.rs`, and
`src/gts.rs` when the design declares GTS types. Transport-agnostic: no DB or
HTTP framework types in the SDK.

## Step 4: Create the gear crate

In order: `Cargo.toml` → `src/lib.rs` (re-export SDK, `pub mod gear;`) →
`src/gear.rs` (`#[toolkit::gear(...)]` + capability impls) → `src/config.rs`
(derive `toolkit_macros::ExpandVars` alongside `Deserialize` — required by
`GearCtx::config_expanded_or_default`) → `domain/` (service, error,
local_client, ports) → `infra/` (entities, migrations, repositories —
through the secure ORM only) → `api/rest/` (dto, handlers, routes) when the
gear has REST. Match the host clippy profile from the start (e.g. unit
structs instead of empty-braced structs — pedantic lints are deny). Tests live in sibling
`*_test.rs` files. A plugin seam scaffolds with its default/noop
implementation. Feature logic stays out — stubs return typed
not-implemented errors.

Tri-backend storage invariants (each learned from a real failure):
- Migration ordering is **lexicographic by name**, not vec order — name
  follow-ups to sort after their predecessors (`initial_002_<topic>`,
  never `<topic>_002`).
- Never `CREATE INDEX IF NOT EXISTS` — MySQL rejects it, and the migration
  table already guards re-runs; plain `CREATE INDEX` works on all three
  backends.
- MySQL column types: `Uuid` fields are `BINARY(16)` (sea-orm/sqlx encode
  `Uuid` as 16 raw bytes — `VARCHAR(36)` breaks on the first insert);
  indexable key columns are `VARCHAR(255)`, never `TEXT`.
- For a gear with the `db` capability, scaffold the Docker DB lane with the
  crates: ignored-by-default `tests/postgres_suite.rs` / `tests/mysql_suite.rs`
  testcontainers suites using the pinned `test-containers` helpers
  (dev-dependencies: `testcontainers-modules`, `test-containers`, and
  `toolkit-db` with the `pg`/`mysql` driver features), following the
  ledger gear's pattern.

## Step 5: Wire and register

Complete the registration surface in the same change set:
- workspace `Cargo.toml`: member paths + versioned path alias
- `apps/cf-gears-example-server/Cargo.toml`: optional dependency + feature
- `apps/cf-gears-example-server/src/registered_gears.rs`: feature-gated `use <crate> as _;`
- workspace `[workspace.metadata.cargo-shear] ignored`: the gear's dependency crates (the macro's hidden re-exports are invisible to cargo-shear; omission breaks main after merge)
- e2e configuration (`config/e2e-local.yaml` / `e2e-features.txt`) — risky external-egress config committed commented out with inline rationale
- the repo gear catalog entry (`docs/GEARS.md`) when the host maintains one
- for a `db` gear: the Makefile DB lane `test-<gear>-db` (nextest over the
  `postgres_*`/`mysql_*` test binaries, `--run-ignored ignored-only`), added
  to the `.PHONY` test list alongside the other per-gear lanes

## Step 6: Scaffold gate

// turbo
Run: `cargo build -p cf-gears-<name> -p cf-gears-<name>-sdk`,
`cargo clippy -p cf-gears-<name> -p cf-gears-<name>-sdk`,
`cargo gears lint --dylint`, `make validate-gear-names`, and
`python3 {scripts}/check_gear_layout.py --repo <host-repo> --gear <name>`
(add `--plugin` for a plugin).
Fix and re-run until all report PASS. Recovery never asks.

## Step 7: Present results

Present the created files, the registration diff, and the gate results.
Name the next step: `cf-gear-implement` against the approved
GEAR-DECOMPOSITION (**user gate — confirmation**).

## Validation Criteria

- [ ] Both crates build and pass clippy and dylint
- [ ] `check_gear_layout.py` reports OK (registration surface complete)
- [ ] `make validate-gear-names` passes
- [ ] No feature logic beyond typed not-implemented stubs

## Next Steps

- `cf-gear-implement` to execute the approved decomposition phases
- `cf-gear-lint` to review lint configuration for the new gear
