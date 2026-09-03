# Decomposition — Audit Log

<!-- toc -->

- [Overview](#overview)
- [Phases](#phases)
  - [Phase 1: SDK Contract](#phase-1-sdk-contract)
  - [Phase 2: Gear Host, Storage, Registration](#phase-2-gear-host-storage-registration)
  - [Phase 3: Query Surface](#phase-3-query-surface)
  - [Phase 4: Retention and End-to-End](#phase-4-retention-and-end-to-end)

<!-- /toc -->

## Overview

Delivery follows the golden path: the SDK contract first so producing gears can compile against the seam, then the gear host with storage and registration, then the query surface, then retention. Each phase closes with the gear-scoped gate; the final phase closes with the full local gate and the end-to-end run. Deferred: the external retention-policy plugin (static config ships first, per `cpt-cf-audit-log-adr-retention-enforcement`).

## Phases

### Phase 1: SDK Contract

- [ ] `p1` - **ID**: `cpt-cf-audit-log-phase-sdk-contract`

#### Scope

`cf-gears-audit-log-sdk`: append/query traits, models, GTS event base type, error catalog. Covers the contract side of `cpt-cf-audit-log-fr-append-event` and `cpt-cf-audit-log-fr-query-trail`.

#### Completion Criteria

- The SDK crate builds and its unit tests pass.
- The GTS base type validates against the type-system validator.
- A consumer can compile a fire-and-forget append call against the trait.

#### Gate

```sh
make gear-ci GEAR=audit-log
```

### Phase 2: Gear Host, Storage, Registration

- [ ] `p1` - **ID**: `cpt-cf-audit-log-phase-host-storage`

#### Scope

Gear crate with macro declaration, scoped storage (`cpt-cf-audit-log-component-storage`), ingestion service (`cpt-cf-audit-log-component-ingestion`) covering `cpt-cf-audit-log-fr-append-event` and `cpt-cf-audit-log-fr-reject-malformed`; full registration surface.

#### Completion Criteria

- The workspace builds with the gear registered in the example server behind its feature flag.
- Ingestion round trip (validate → durable insert → ack) passes gear-scoped tests, including a malformed-event rejection.
- Layout and name checks pass.

#### Gate

```sh
make gear-ci GEAR=audit-log && make validate-gear-names
```

### Phase 3: Query Surface

- [ ] `p1` - **ID**: `cpt-cf-audit-log-phase-query`

#### Scope

Query service (`cpt-cf-audit-log-component-query`) with OData-filtered REST covering `cpt-cf-audit-log-fr-query-trail`; per-query authorization.

#### Completion Criteria

- Filtered queries return scoped results in gear-scoped tests.
- An unauthorized query is refused with the typed error.

#### Gate

```sh
make gear-ci GEAR=audit-log
```

### Phase 4: Retention and End-to-End

- [ ] `p2` - **ID**: `cpt-cf-audit-log-phase-retention-e2e`

#### Scope

Retention task (`cpt-cf-audit-log-component-retention`) covering `cpt-cf-audit-log-fr-enforce-retention`, with the deletion event re-entering ingestion; end-to-end configuration.

#### Completion Criteria

- A retention sweep deletes expired records and its deletion event appears in the trail, in tests.
- The full local gate passes; the end-to-end suite runs the append→query→retain journey.

#### Gate

```sh
make check && make gear-ci GEAR=audit-log && python3 tools/scripts/run_e2e.py --features audit-log
```
