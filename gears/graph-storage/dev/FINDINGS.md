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
