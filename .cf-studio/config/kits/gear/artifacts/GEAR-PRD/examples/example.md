# PRD — Audit Log

<!-- toc -->

- [1. Overview](#1-overview)
  - [1.1 Purpose](#11-purpose)
  - [1.2 Background / Problem Statement](#12-background--problem-statement)
  - [1.3 Goals (Business Outcomes)](#13-goals-business-outcomes)
- [2. Gear Classification](#2-gear-classification)
- [3. Actors](#3-actors)
- [4. Scope](#4-scope)
  - [4.1 In Scope](#41-in-scope)
  - [4.2 Out of Scope](#42-out-of-scope)
- [5. Functional Requirements](#5-functional-requirements)
  - [5.1 Ingestion](#51-ingestion)
  - [5.2 Query](#52-query)
  - [5.3 Retention](#53-retention)
- [6. Non-Functional Requirements](#6-non-functional-requirements)
- [7. Public Interfaces](#7-public-interfaces)
- [8. Use Cases](#8-use-cases)
- [9. Acceptance Criteria](#9-acceptance-criteria)
- [10. Dependencies](#10-dependencies)
- [11. Risks](#11-risks)

<!-- /toc -->

## 1. Overview

### 1.1 Purpose

The audit-log gear owns the platform's tamper-evident record of privileged actions: who did what, in which tenant, to which resource, when, and with what outcome. It ingests audit events from other gears and exposes tenant-scoped queries for support and compliance.

### 1.2 Background / Problem Statement

Today each gear logs free-form text; answering "who deleted this credential" means grepping pods, and retention is uncontrolled. See GEAR-INTENT `cpt-cf-audit-log-intent-tenant-audit-trail`.

### 1.3 Goals (Business Outcomes)

- A support engineer answers a who-did-what question for any covered action within one minute, using one API.
- Compliance retention is provable per tenant: records exist exactly as long as policy demands.

## 2. Gear Classification

**Intent**: `cpt-cf-audit-log-intent-tenant-audit-trail`

| Attribute | Value |
|-----------|-------|
| Tier | service |
| Capabilities | rest, db |
| Archetype | REST+DB service gear |
| Reference gear | simple-user-settings (layering), usage-collector (ingestion seam) |
| Similar gears | usage-collector (aggregates, not verbatim), event-broker (transport, reused), file-storage (blob semantics, not a match) |

## 3. Actors

#### Support Engineer

**ID**: `cpt-cf-audit-log-actor-support-engineer`

**Role**: Queries audit records to answer who-did-what questions.
**Needs**: Fast, tenant-scoped, filterable queries.

#### Producing Gear

**ID**: `cpt-cf-audit-log-actor-producing-gear`

**Role**: Any platform gear emitting audit events for its privileged actions.
**Needs**: A fire-and-forget append API that never blocks its own operations.

## 4. Scope

### 4.1 In Scope

- Append-only ingestion of typed audit events from producing gears.
- Tenant-scoped query with filtering by actor, resource, action, and time.
- Per-tenant retention enforcement.

### 4.2 Out of Scope

- Usage metering and aggregation — owned by usage-collector.
- Cross-gear event distribution — owned by event-broker.
- Log shipping to external SIEM systems — future exporter plugin.

## 5. Functional Requirements

### 5.1 Ingestion

#### Append Audit Event

- [ ] `p1` - **ID**: `cpt-cf-audit-log-fr-append-event`

The gear **MUST** accept a typed audit event from a producing gear and durably append it to the tenant's trail, acknowledging only after durability.

**Rationale**: The trail is only trustworthy if acknowledged events cannot be lost.

**Actors**: `cpt-cf-audit-log-actor-producing-gear`

#### Reject Malformed Events

- [ ] `p1` - **ID**: `cpt-cf-audit-log-fr-reject-malformed`

The gear **MUST** reject events that fail type validation, identifying the violated type to the producer.

**Rationale**: A trail of unvalidated events cannot support compliance claims.

**Actors**: `cpt-cf-audit-log-actor-producing-gear`

### 5.2 Query

#### Query Trail

- [ ] `p1` - **ID**: `cpt-cf-audit-log-fr-query-trail`

The gear **MUST** answer tenant-scoped queries filtered by actor, resource, action, and time range. A support engineer may query only tenants they are authorized for; producing gears may not query.

**Rationale**: The one-minute support answer is the gear's reason to exist.

**Actors**: `cpt-cf-audit-log-actor-support-engineer`

### 5.3 Retention

#### Enforce Retention

- [ ] `p2` - **ID**: `cpt-cf-audit-log-fr-enforce-retention`

The gear **MUST** delete a tenant's records after that tenant's retention period elapses, and **MUST** record the deletion itself as an audit event.

**Rationale**: Retention is a compliance obligation in both directions — keep long enough, and no longer.

**Actors**: `cpt-cf-audit-log-actor-support-engineer`

## 6. Non-Functional Requirements

#### Ingestion Latency Isolation

- [ ] `p1` - **ID**: `cpt-cf-audit-log-nfr-ingestion-isolation`

Producing gears **MUST NOT** be delayed by audit ingestion beyond a bounded acknowledgment budget.

**Threshold**: p99 append acknowledgment under 50 ms at nominal load.

**Rationale**: A trail that slows producers gets bypassed.

## 7. Public Interfaces

- SDK: consumed by producing gears for typed append; consumed by support tooling for queries.
- REST: present, for support-facing queries.
- GTS extension points: audit event types are GTS-derivable — producing gears register their own event types.
- Plugin seam: retention policy source (static config vs external policy service).

## 8. Use Cases

#### Answer a Who-Did-What Question

- [ ] `p2` - **ID**: `cpt-cf-audit-log-usecase-who-did-what`

**Actor**: `cpt-cf-audit-log-actor-support-engineer`

**Main Flow**:
1. Engineer queries the tenant's trail filtered by resource and time range.
2. The gear returns matching events with actor, action, outcome, and timestamps.

**Postconditions**:
- The question is answered from one API without host access.

## 9. Acceptance Criteria

- [ ] A covered privileged action in a producing gear appears in the trail and is queryable within seconds.
- [ ] A tenant's records disappear after its retention period, and the deletion is itself recorded.

## 10. Dependencies

| Dependency | Description | Criticality |
|------------|-------------|-------------|
| authz-resolver | Query authorization decisions | p1 |
| types-registry | Registration of audit event GTS types | p1 |
| tenant-resolver | Tenant scoping of trails and retention | p1 |

## 11. Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Producers bypass the trail under load | Compliance gaps | Bounded-latency NFR; fire-and-forget SDK path |
| Event type sprawl | Unqueryable trail | GTS-derived event types with registration review |
