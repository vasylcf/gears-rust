# CF/Gears Kit (Constructor Studio-compatible)

**ID**: `gears`
**Format**: `Constructor Studio` (layout-only, deprecated for gear creation)
**Purpose**: Bind CF/Gears's documentation templates (`docs/spec-templates/*`)
and expert checklists (`docs/checklists/*`) as artifact kinds for the
**already-registered** gear documentation, and host the PR-review tooling.

> **Gear creation moved.** The end-to-end gear-creation path — intake,
> GEAR-PRD/DESIGN/ADR/FEATURE authoring, decomposition, scaffold, phased
> implementation with gates, and PR assembly — is provided by the
> [`gear` kit](https://github.com/vasylcf/kit-gear-creation) (installed under
> `.cf-studio/config/kits/gear`, skills `cf-gear-*`). The document-authoring,
> decomposition, implementation, and coding workflows this kit used to carry
> were removed in its 0.2.0 release; this kit stays registered as the kind
> provider for existing `gears/*/docs` artifacts until they migrate to the
> GEAR-* kinds, and as the home of the PR-review automation below.

## Artifact kinds (existing documents)

| Kind | Template source | Checklist source |
|------|------------------|------------------|
| PRD | `docs/spec-templates/PRD.md` | `docs/checklists/PRD.md` |
| DESIGN | `docs/spec-templates/DESIGN.md` | `docs/checklists/DESIGN.md` |
| ADR | `docs/spec-templates/ADR.md` | `docs/checklists/ADR.md` |
| FEATURE | `docs/spec-templates/FEATURE.md` | `docs/checklists/FEATURE.md` |
| DECOMPOSITION | `docs/spec-templates/DECOMPOSITION.md` | `docs/checklists/DECOMPOSITION.md` |

## Remaining workflows

- `cf-gears-pr-review`: LLM PR review via `scripts/pr.py`, reports to `.prs/<ID>/`.
- `cf-gears-pr-status`: PR status reports and comment-severity audit.
- `cf-gears-change-impact-analysis`: upstream artifact change → downstream artifacts/code.
- `cf-gears-doc-upstream-reqs`: UPSTREAM_REQS authoring (no gear-kit counterpart yet).
