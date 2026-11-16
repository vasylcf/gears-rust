---
status: accepted
date: 2026-11-16
decision-makers: Constructor Fabric Steering Committee
---

# Configuration Read-Through — No Control Plane Cache

**Supersedes**: [ADR-0007: Control Plane Caching](./0007-data-plane-caching.md).

## Context and Problem Statement

The multi-layer L1/L2 configuration cache of ADR-0007 was never built, and keeping two cache
layers coherent across replicas costs more than the repository reads it would save.

## Decision Outcome

OAGW reads upstream and route configuration from the repositories on every proxied request.
There is no configuration cache of any kind. Endpoint connection pools stay cached.
