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

## F9 — the single-statement hop is only a win in one query shape

A local Level A implementation (`libs/toolkit-db/src/secure/cte.rs`) was built to
measure what a safe CTE actually buys the gear. Two things had to be fixed
before the comparison meant anything, and both are findings in their own right.

**A CTE body selects `*` unless told otherwise.** A CTE referenced more than
once is materialised by PostgreSQL, so a body of `SELECT *` materialises the
edge table's `payload` jsonb on every hop. The two-query hop never had this
problem because `project_all` already narrows the projection. The CTE API needed
the same escape hatch, added as `into_cte_projected` / `project_with_ctes`.

**`id IN (a) OR id IN (b)` costs a sequential scan.** The natural way to say
"either endpoint of an incident edge" is two `IN` subqueries joined by `OR`.
PostgreSQL cannot drive an index from two hashed subplans under an `OR`, so it
falls back to a seq scan of `graph_node` — 198 993 rows removed to return 11:

| outer predicate | plan | execution |
|---|---|---|
| `id IN (src) OR id IN (dst)` | Seq Scan on `graph_node` | 15.18 ms |
| `id IN (SELECT src UNION SELECT dst)` | Nested Loop + Index Only Scan | 0.30 ms |

Same rows, 50x apart. The union form is now the only one the gear can build:
`cte_columns_union` in `toolkit-db` emits it, and
`the_outer_query_probes_the_node_table_once` fails if the hop regresses to the
`OR` shape.

**Measured, once both were fixed** — 199 004 nodes / 599 999 edges, hub-skewed
degree distribution, 40 fixed seeds, debug build, end-to-end over HTTP:

| depth | two scoped queries | one scoped CTE |
|---|---|---|
| 1 | p50 4.1 / p95 4.7 / p99 5.1 ms | p50 3.7 / p95 4.2 / p99 4.5 ms |
| 2 | p50 6.6 / p95 8.0 / p99 8.9 ms | p50 5.5 / p95 6.8 / p99 7.8 ms |
| 3 | p50 13.9 / p95 50.5 / p99 66.6 ms | p50 10.7 / p95 30.0 / p99 34.2 ms |

Results are identical across all 120 queries, and the cross-tenant trap fixture
(F1) yields `{1,2}` at depth 1 and `{1,2,3}` at depth 2 under both strategies.

The gain is real but modest at shallow depth — roughly 10–15 % — and grows at
the tail, where depth 3 p95 drops by 41 %. That tail is where the two-query hop
pays for candidate lists that cross the process boundary: on a hub-heavy
frontier the intermediate endpoint set is large, and shipping it to the gear
only to ship it back as an `IN` list is the dominant cost. The stand runs over
loopback, so the extra round trip itself is nearly free (~0.05 ms); on a
1 ms-RTT network the shallow-depth ranking inverts and the deep-depth gap widens.

Conclusion for the toolkit-db discussion: a safe CTE primitive is worth having,
but the case for it is **tail latency and payload volume on wide frontiers**,
not per-hop overhead. Correctness never depended on it — see F1.

## F10 — SQL/PGQ needs no `sea_query` patch, and PGQ is a catalog rewrite

Reaching `GRAPH_TABLE` from Rust looked like it required forking `sea_query` to
add an AST node for it. It does not. Three existing pieces compose:

- `TableRef::FunctionCall` puts a call in the `FROM` clause;
- `Func::Custom` renders the name **raw and unquoted**
  (`sea-query-1.0.2/src/backend/query_builder.rs:768`) — load-bearing, because
  `GRAPH_TABLE(...)` parses and `"GRAPH_TABLE"(...)` is a syntax error;
- `Expr::cust_with_values` renders arbitrary text while **binding** its values.

Composed, they emit exactly the statement the stand executes:

```sql
SELECT "neighbour" FROM GRAPH_TABLE(kb_pgq
  MATCH (a IS node)-[e IS edge]->(b IS node)
  WHERE a.id = $1 AND a.tenant_id = $2 AND b.tenant_id = $3
  COLUMNS (b.id AS neighbour)) AS "g"
```

Both guarantees are pinned by tests and were verified to fail when broken:
quoting the construct name makes PostgreSQL reject the statement, and dropping
the tenant predicate makes a foreign tenant read **2 rows** it does not own.

So the obstacle to SQL/PGQ is not `sea_query`. It is that `Expr::cust` is raw
SQL, which gear code may not write — a policy question whose answer is that the
construct belongs inside `toolkit-db`, where the platform CTE policy already
exempts dialect-specific assembly (the outbox writer precedent).

### What `CREATE PROPERTY GRAPH` actually is

Read off the stand's catalogs, because the "it works like a view" summary is
close but misleading in the part that matters.

A property graph is a relation with **no storage**: `relkind = 'g'`,
`relnatts = 0`. Like a view it occupies a name in `pg_class` and holds no rows.
Unlike a view — which stores one query's parse tree in `pg_rewrite` — it stores
structured metadata across five catalogs. Ours holds:

| alias | table | kind | key | srckey → srcref | destkey → destref |
|---|---|---|---|---|---|
| graph_node | graph_node | v | {1,2} | | |
| graph_edge | graph_edge | e | {1,2} | {1,5} → {1,2} | {1,6} → {1,2} |

Those are `attnum`s: `{1,2}` is `(tenant_id, id)`, `{1,5}` is
`(tenant_id, src_node_id)`. The catalog records the join, nothing more.

`GRAPH_TABLE` is expanded at parse analysis into that join. The one-hop pattern
plans as `graph_node ⋈ graph_edge ⋈ graph_node` with
`Index Cond: (tenant_id = graph_edge.tenant_id AND id = graph_edge.dst_node_id)`
— a tenant equality nobody wrote, derived from the composite element keys.

Four consequences worth stating plainly:

- **The abstraction is free at runtime.** After expansion it is an ordinary
  plan: same indexes, same statistics, same `EXPLAIN`, same RLS.
- **There is no second store.** No dual write, no sync, no drift. Recreating the
  property graph is a catalog operation.
- **The tenant join is structural**, not written by us — but it only stops an
  edge from *reaching* a foreign node. A pattern with no tenant predicate still
  *returns* every tenant's rows, which the mutation above demonstrates.
- **It is not a graph engine.** No adjacency list, no traversal operator; every
  hop is a join. That is why multi-hop chain patterns explode on hubs, and why
  variable-length quantifiers are absent in PostgreSQL 19 — variable depth
  cannot expand into a fixed number of joins.

## F11 — the undirected shorthand is a 2350x trap, and the tenant predicate is not symmetric

Two measurements taken while building the typed pattern builder. Both change
what the builder is allowed to express, so both are recorded rather than left
as folklore.

### `(a)-[e]-(b)` is not shorthand, it is a full scan

The spike already said the undirected shorthand "plans as an all-vertex probe".
The size of that on real data was not measured until now. Same seed, same 10
rows:

| pattern | plan | execution |
|---|---|---|
| `(a)-[e]-(b)` | Parallel Seq Scan on `graph_edge` | 734.9 ms |
| two directed patterns, `UNION` | two index scans on `idx_graph_edge_src` / `_dst` | 0.312 ms |

So the builder has no undirected variant at all. [`Direction`] offers
`Outgoing` and `Incoming`; an undirected hop is two patterns. The convenience
form is not a slower way to write the same thing — it is a different query.

### Either endpoint predicate fences the tenant; neither is optional

For a one-hop pattern seeded by id, measured on the stand:

| tenant predicate | foreign tenant sees | own tenant sees |
|---|---|---|
| source endpoint only | 0 | 2 |
| target endpoint only | 0 | 2 |
| none | **2** | 2 |

Either endpoint alone anchors the walk, because composite element keys tie both
ends of an edge to one tenant. What is not optional is having a predicate at
all: with none, a caller who names an id reads whichever tenant owns it.

The consequence for testing is that no execution test can detect the loss of
*one* of the two predicates — both mutations pass the stand suite. The builder
emits both anyway (one bound value, no plan change, and it stays correct if the
element keys ever stop carrying `tenant_id`), and the guard for that redundancy
is a unit test on the emitted text rather than a behavioural one.

### What the builder guarantees

Nothing reaching the pattern text is a caller string. Identifiers come from
closed enums (`Graph`, `Label`, `Var`, `Property`, `Output`); values are bound;
a frontier of any size binds as **one** parameter (`= ANY($n::bigint[])`), so
the statement text does not vary with the number of seeds; and the tenant is a
constructor argument rather than a predicate the caller may omit. Verified by
mutation: dropping both tenant predicates makes a foreign tenant read 2 rows,
rendering both directions with the same arrow makes the two directions return
the same set, and moving the edge-type restriction to the wrong variable fails
the emitted-SQL assertion.

