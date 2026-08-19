# ADR-0002 probe — results

Checking the proposals in [secure-orm ADR-0002](https://github.com/constructorfabric/gears-rust/pull/4605)
against a running PostgreSQL 19 beta2 with a property graph over real tables,
rather than against reasoning. Scripts in this directory reproduce every number.

Fixture: 199k nodes / 600k edges, out-degree ~3, destination-skewed (top node
has 3449 incoming edges). Composite element keys `(tenant_id, id)`.

## Confirmed, without qualification

**Element-level `WHERE` works.** `(a IS node WHERE a.tenant_id = $1)` executes,
which is the mechanism the whole ADR rests on.

**Bound parameters work inside an element `WHERE`,** and one placeholder can be
referenced from several elements. The renderer will not have to inline values.

**Policy 3's rationale is real.** A column absent from the `PROPERTIES` list is
invisible to the pattern language:

```
ERROR:  property "name" does not exist
```

So a property graph whose DDL omits the scope columns cannot be scope-filtered
at all — exactly the ADR's argument for generating DDL and query from one
declaration. This gear's own DDL happens to list `tenant_id`; had it not, none
of its scoping would work.

**Policy 2's hazard is real and silent.** `WHERE false` inside an element
pattern parses and returns zero rows, so an element table that resolves no scope
property produces an empty traversal that looks like an empty neighbourhood.

## Phase A — the mandate costs nothing

ADR-0002 requires the scope in *every* element pattern. Measured against
carrying it in the pattern's top-level `WHERE`, which is what this gear ships:

| form | median | plan |
|---|---|---|
| top-level `WHERE` | 0.215 ms | Nested Loop, index scans |
| per-element `WHERE` (ADR-0002) | 0.212 ms | **identical** |
| per-element, edge unscoped | 0.170 ms | identical shape |

Same rows, same plan — PostgreSQL normalises the two forms. The mandate is free.
Scoping the edge element as well costs about 0.04 ms here, absorbed into the
composite index condition; it is not a reason to skip it.

## Phase B — the multi-hop story is about direction, not about paths

The ADR's `PathBuilder` reads as a path, so the natural way to ask for a
two-hop neighbourhood is one `MATCH` with several elements. The PG19 spike had
recorded that shape as unusable ("exact-3-hop took 24.4 s"). On this schema that
turns out to be **too broad, and the real cause is narrower**.

Directed path patterns against chained one-hop patterns, both with ADR-0002
per-element scoping, same rows returned in every case:

| seed (out-degree) | depth | one path pattern | chained one-hop |
|---|---|---|---|
| 1875 (13) | 2 | 1.461 ms | 1.529 ms |
| 1875 (13) | 3 | 3.309 ms | 3.377 ms |
| 5000 (2) | 2 | 0.336 ms | 0.459 ms |
| 5000 (2) | 3 | 1.357 ms | 0.950 ms |

No penalty. Even walking *backwards* through the destination hub, where path
multiplication should show, it barely does: 9983 rows for 9650 distinct nodes —
3.4 % redundancy, 787 ms, and the size comes from the neighbourhood genuinely
being that large.

What does explode is **undirected** elements, and it compounds per element:

| pattern | rows | time |
|---|---|---|
| one undirected element | 10 | 734.9 ms |
| two undirected elements | 83 | **7 967 ms** |
| the same two, directed | 83 | ~1.5 ms |

An undirected element plans as a probe over every vertex, so a second one
multiplies that. This is the argument for direction being explicit in
`PathBuilder`: not a 2350x penalty at one hop, but a penalty that compounds with
path length.

**Caveat on generality.** This graph has out-degree ~3. Path multiplication
grows with branching factor, so a denser graph would show a gap between the path
form and the chained form that this fixture does not. The direction result is
not fixture-dependent in the same way — an all-vertex probe is an all-vertex
probe at any density.

## Phase C — one scope compiler does serve both paths, with one qualification

ADR-0002 says the only substantive change the Secure ORM core needs is to
parameterise *how a resolved column is addressed*, and explicitly rejects
writing a second PGQ-specific scope compiler. That is implemented here as
`ColumnAddress::{Table, GraphElement(var)}` plus
`build_scope_condition_addressed`, in `libs/toolkit-db/src/secure/cond.rs`.

**The claim holds.** Table addressing reproduces the original compiler's output
exactly — asserted by rendering both and comparing, across tenant scopes,
`deny_all`, `allow_all` and a tenant-subtree scope. Graph addressing emits
`"dst"."tenant_id"` instead of `"custom_prop_test"."tenant_id"`, and nothing
else changes.

**And the emitted predicate executes.** The compiler's output was spliced
verbatim into an element pattern on PG19 beta2:

```sql
MATCH (a IS node WHERE "a"."tenant_id" IN ('...') AND a.id = 5000)
     -[e IS edge]->
      (b IS node WHERE "b"."tenant_id" IN ('...'))
```

It returned the owning tenant's two neighbours, and zero rows for a foreign
tenant. Quoted identifiers are accepted inside `MATCH`, so `sea_query`'s
escaping path — which ADR-0002 requires identifiers to go through — does not
have to be bypassed.

**The qualification the ADR's diagram does not carry.** Addressing is necessary
but not sufficient. Three `ScopeFilter` arms compile to `col IN (SELECT …)`, and
PG19 rejects a subquery inside a pattern, so in graph mode those arms have to
**fail**, not render. They must also fail *loudly*: dropping a filter is
fail-closed in the letter — the constraint vanishes and the scope compiles to
`WHERE false` — and that is precisely the silent empty traversal Policy 2 exists
to prevent. Implemented as `AddressError::SubqueryInPattern`, returned rather
than swallowed, and pinned by a test asserting the same scope is still fine
against a table.

So the shape the core needs is not
`build_scope_condition(scope, addressing) -> Condition`
but
`build_scope_condition(scope, addressing) -> Result<Condition, _>`,
which is a signature change rather than an internal one.

## Phase D — Policy 2 has to be a policy; the compiler cannot catch it

ADR-0002 requires every element table to resolve at least one scope property,
and says the builder must reject one that cannot rather than emit a deny-all
traversal. That is the right call, and this shows why it cannot be delegated to
the scope compiler.

An entity resolving nothing — the shape `tenant_closure`,
`resource_group_membership` and the rest have today — compiles to exactly the
same condition under a real tenant scope as it does under `deny_all`:

| entity, scope | rendered |
|---|---|
| resolves nothing, `for_tenant(t)` | `WHERE FALSE` |
| resolves nothing, `deny_all()` | `WHERE FALSE` |

The two are indistinguishable at the point where the condition is built, because
the failure happens inside `resolve_property` rather than in a filter arm. So
"is this an eligible graph element" is a question about the *entity*, asked
before compiling, and it belongs where the ADR puts it — in the builder's
admission of an element, not in the scope compiler.

## Phase E — the subquery fallback is expressible, but not through this API

The third fallback offered on the PR — keep the closure query in the same
statement and correlate the pattern against it — needs two things: a second
`FROM` item, and a pattern predicate that references its alias. Both are
accepted by PostgreSQL:

| shape | works |
|---|---|
| `FROM graph_node n, GRAPH_TABLE(… WHERE a.id = n.id …)` | yes |
| `FROM (SELECT …) s, GRAPH_TABLE(… WHERE a.id = s.id …)` | yes |

The first is the more interesting one for ADR-0002. `with_graph()` starts from a
`SecureSelect<E, Scoped>` and keeps outer-query operations, so `E`'s table is
presumably in the `FROM` — which means a pattern could correlate against the
outer entity's own columns, and a traversal could be seeded from a scoped entity
query with no new construct at all. Whether `PathBuilder` can express a
predicate referring to the outer alias is an API question, not a SQL one.

What the API has no room for is an **arbitrary** sibling. A tenant-subtree scope
needs the closure as a correlation source, and the closure is neither `E` nor a
graph element — and it cannot be the outer entity either, because closure tables
resolve no scope property and a scoped select over one compiles to `WHERE false`
(Phase D). So the fallback needs either an API addition along the lines of
`.with_source(name, subquery)`, or the closure pre-resolved after all.

That is worth settling alongside the open question rather than after it: if the
answer to "may a pattern hold a subquery" is no — and it is — then whether the
API can hold a sibling decides whether subtree scopes are servable at all, or
whether fallback 1 (reject) is where v1 lands.

