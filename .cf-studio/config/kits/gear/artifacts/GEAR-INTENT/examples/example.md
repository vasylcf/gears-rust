# GEAR-INTENT — Tenant Audit Trail


<!-- toc -->

- [Request](#request)
- [Classification](#classification)
- [Similar Gears](#similar-gears)
- [Reuse Profile](#reuse-profile)
- [Document Set](#document-set)
- [Route Decision](#route-decision)

<!-- /toc -->

**ID**: `cpt-cf-audit-log-intent-tenant-audit-trail`

**Date**: 2026-08-20
**Requester**: platform PM, issue gears-rust#9999

## Request

Product operations needs a tamper-evident record of who did what, in which tenant, to which resource. Today each gear logs free-form text; support cannot answer "who deleted this credential" without grepping pods. The desired capability is a queryable, tenant-scoped audit trail every gear can write to, with retention controls. Acceptance outcome: a support engineer answers a who-did-what question for any covered action within one minute, using one API.

## Classification

**Class**: new gear

Audit recording is a cross-cutting capability no existing gear owns: it has its own data lifecycle (append-only, retention), its own consumers (support, compliance), and its own API surface. It is not a plugin — there is no host gear whose seam covers append-only tenant-scoped storage with retention. It is not a small fix — it introduces new public contracts.

## Similar Gears

| Gear | Tier / status | Overlap | Verdict |
|------|---------------|---------|---------|
| usage-collector | system, implemented | ingests per-tenant usage events via plugins | not a match — usage aggregation folds events; audit needs verbatim append-only records |
| event-broker | system, implemented | distributes events between gears | reuse as transport — audit can consume broker events, not replace it |
| file-storage | service, implemented | tenant-scoped persistence | not a match — blob semantics, no query-by-actor |

**Reuse-or-build call**: build new — no existing gear owns append-only audit semantics; reuse event-broker as an ingestion path and imitate usage-collector's plugin-based ingestion shape.

## Reuse Profile

**Archetype**: REST+DB service gear

**Reference gear to imitate**: simple-user-settings for the minimal REST+DB layering; usage-collector for the ingestion plugin seam.

**Core toolkit libraries**: toolkit, toolkit-macros, toolkit-security, toolkit-gts, toolkit-canonical-errors, toolkit-db, toolkit-db-macros, toolkit-odata

**Optional toolkit libraries**: none required at intake; revisit at design for export pagination.

**Platform dependencies**: authz-resolver (query authorization), types-registry (GTS event types), tenant-resolver (tenant scoping)

## Document Set

| Document | Needed | Reason |
|----------|--------|--------|
| GEAR-PRD | yes | new gear with new public surface |
| GEAR-DESIGN | yes | storage model, ingestion seam, query API need allocation |
| GEAR-ADR | as decisions arise | expected: retention enforcement, ingestion path (direct vs broker) |
| GEAR-DECOMPOSITION | yes | delivery spans SDK, storage, ingestion, query phases |
| GEAR-FEATURE | optional | only if the retention scheduler proves complex |

## Route Decision

**Route**: doc-prd → doc-design → doc-adr (retention, ingestion) → decompose → scaffold → implement → pr

**First step**: invoke the GEAR-PRD authoring workflow with this intent as input.

**Confirmed by**: platform PM, 2026-08-20
