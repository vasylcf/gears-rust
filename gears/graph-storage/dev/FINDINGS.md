# Development stand findings

Observations produced by running the gear against a live PostgreSQL 19 beta2,
rather than by reading the platform sources. Each entry states what was tried,
what happened, and what it means for the design.

## F1. A gear cannot scope a subquery, a join, or a CTE body

**Tried.** The natural single-statement hop: a scoped node query whose predicate
contains a subquery over the edge table.

```sql
SELECT n.id FROM graph_node n
WHERE <node scope>
  AND n.id IN (SELECT dst_node_id FROM graph_edge WHERE src_node_id = ANY($1))
```

**Happened.** The outer query is scoped by the secure ORM. The inner one cannot
be: the only way to obtain a scope predicate as a reusable `Condition` is
`build_scope_condition`, and while it is declared `pub`
(`libs/toolkit-db/src/secure/cond.rs:54`) it lives in a private module —
`libs/toolkit-db/src/secure/mod.rs:106` declares `mod cond;`, not `pub mod cond`.
A gear cannot name it, so the subquery ships unscoped.

Pinned by `traversal_experiment::tests::gear_built_edge_subquery_has_no_tenant_predicate`,
which asserts the emitted SQL contains the frontier predicate and no tenant
predicate. When that test starts failing, a scoped custom-query primitive has
landed and the production traversal can collapse from two queries into one.

**Why it matters here specifically.** Surrogate ids are allocated per tenant, so
`src_node_id = ANY($1)` matches edges in *every* tenant that happens to have a
node with that id. An unscoped edge subquery would make the walk follow foreign
edges. The stand reproduces this: our tenant holds `1 -> 2 -> 3`, a foreign
tenant holds `1 -> 3` under the same surrogate ids.

## F2. The bounded hop is expressible today — as two scoped queries

**Tried.** Splitting the hop: one scoped query over edges, then one scoped query
over the endpoints it produced.

**Happened.** Works, and stays inside the secure ORM with no raw SQL anywhere in
the gear. Verified against the trap above:

| Request | Result | Note |
|---|---|---|
| `seeds=1&depth=1` | `{1, 2}` | node 3 is reachable only via the foreign edge and is not returned |
| `seeds=1&depth=2` | `{1, 2, 3}` | node 3 arrives through our own `2 -> 3` |

**Cost.** Two round trips per hop instead of one, so a depth-3 request issues six
queries rather than three. Both are bounded and indexed, so this is a constant
factor rather than a scaling problem — but it is the concrete price of the
missing primitive, and it is what a CTE-based hop would remove.

**Consequence for the design.** The traversal port can ship without any new
platform capability. The single-statement composition that CTEs and SQL/PGQ
unlock — vector KNN, graph expansion and full-text in one statement — remains
the reason to want them, not basic correctness.

## F3. Composite keys and BIGINT ids are compatible with the scope machinery

`#[derive(Scopable)]` maps authorization properties to columns; it says nothing
about the primary key. Composite `PRIMARY KEY (tenant_id, id)` and two
`#[sea_orm(primary_key)]` fields work unchanged, and BIGINT surrogate ids
convert to `ScopeValue::Int`
(`libs/toolkit-db/src/secure/db_ops.rs`), so the partition-ready key contract
from the design costs nothing at the ORM layer.

## F4. Platform details worth knowing before the next gear

* A gear receives its migrations only if it actually acquires the database in
  `init` (`ctx.db_required()`); declaring the `db` capability alone is silently
  insufficient.
* The configuration needs a top-level `database:` section. A DSN placed only
  inside the gear entry fails with `No global database section found`.
* With `auth_disabled`, the injected root context carries
  `DEFAULT_TENANT_ID = 00000000-df51-5b42-9538-d2b56b7ee953`, not a nil UUID.
* `#[resource_error]` generates exactly the canonical categories — `aborted`,
  `already_exists`, `cancelled`, `data_loss`, `deadline_exceeded`,
  `failed_precondition`, `invalid_argument`, `not_found`, `out_of_range`,
  `permission_denied`, `resource_exhausted`, `unimplemented`, `unknown`. There is
  no `internal`; the design's error matrix should use `unknown` for that row.

## F5. SQL/PGQ accepts composite keys, and they fence tenants structurally

`CREATE PROPERTY GRAPH` takes composite element keys, so the partition-ready
schema needs no special handling:

```sql
graph_edge KEY (tenant_id, id)
  SOURCE KEY (tenant_id, src_node_id) REFERENCES graph_node (tenant_id, id)
```

The consequence is stronger than compatibility. Because the source and
destination keys carry `tenant_id`, an edge cannot join a node of another
tenant, so **no pattern can cross a tenant boundary even before a scope
predicate is applied**. With the cross-tenant id-collision fixture from F1 in
place, a pattern seeded on our tenant's node 1 returns only node 2; the foreign
`1 -> 3` edge is unreachable by construction rather than by filtering.

This does not remove the need for the caller's scope — a query with no tenant
predicate still returns rows from every tenant, each internally consistent — but
it removes the class of error where a walk silently follows a foreign edge.

Gotcha worth recording: `REFERENCES` names the graph *element*, which defaults
to the table name, not the `LABEL`. `REFERENCES node (...)` fails with
"source vertex node of edge graph_edge does not exist" when the table is
`graph_node`.

## F6. Hop cost: two scoped queries are not the slow option

One undirected hop, 200,003 nodes / 600,000 edges (hub-skewed destinations),
random seeds, single client, 20 s pgbench runs, per-transaction latency log:

| Shape | p50 | p95 | p99 |
|---|---|---|---|
| Two scoped queries (what the gear does) | 0.183 ms | 0.371 ms | 0.688 ms |
| Single statement with a scoped CTE | 0.213 ms | 0.432 ms | 0.811 ms |
| SQL/PGQ `GRAPH_TABLE`, direction-explicit union | 0.402 ms | 0.645 ms | 1.082 ms |

Two plain indexed lookups beat one CTE statement, and SQL/PGQ costs about 1.7x
the plain-SQL hop.

**Caveat that matters.** These run over a loopback socket, where a round trip is
roughly 0.05 ms. The two-query shape spends one extra round trip per hop, so on
a network with a 1 ms round trip it would lose to the single-statement shapes
by about 1 ms per hop while the SQL-level difference stays under 0.3 ms. The
ranking above is therefore a property of this deployment, not of the shapes.
What the numbers do settle is that no shape is disqualified on cost.

## F7. End-to-end traversal against the NFR

`GET /neighbours` on the same graph, measured through HTTP, **debug build**, with
a deliberately naive visited set (linear `contains` rather than a hash set):

| Depth | p50 | p95 | max |
|---|---|---|---|
| 1 | 8 ms | 10 ms | 17 ms |
| 2 | 10 ms | 18 ms | 101 ms |
| 3 | 24 ms | 89 ms | 137 ms |

The traversal NFR allows 1 s at p95 for depth 3 on a 100k-node / 500k-edge
profile. This fixture is larger, the build is unoptimised and the implementation
is the naive one, and it still lands an order of magnitude inside the budget.

This is not yet the full NFR claim: the endpoint returns bare node ids, without
degree ordering, phantom filtering, hydration or metric annotations. It does
cover bounded expansion, budget enforcement and authorization, which is the part
the earlier PG19 spike explicitly could not measure.

## F8. Platform migrations can create a property graph

`CREATE PROPERTY GRAPH` runs cleanly through the platform migration runner —
DDL beyond `CREATE TABLE` is not a problem for it. The stand's second migration
creates `kb_pgq` on every fresh database, so the SQL/PGQ backend has something to
target without manual setup.
