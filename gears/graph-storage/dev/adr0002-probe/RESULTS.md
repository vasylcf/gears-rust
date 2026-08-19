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
