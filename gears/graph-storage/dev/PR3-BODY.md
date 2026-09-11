# The graph-storage gear

The implementation of the documents merged in #4523. That PR carried the PRD,
the DESIGN and the ADRs and no code; this is the code, plus the amendments
building it forced on those documents.

## Read the documentation changes first

Every place where building the thing contradicted the design is amended in
place and marked **"Found while building the prototype"**, so a reader can
tell a decision taken up front from one the implementation forced. The
substantive ones:

- **ADR-0006 (new, `accepted`): a backward-compatible type change is admitted
  under the same GTS identifier.** The documents said a changed schema under a
  registered identifier is a conflict, full stop. types-registry ADR-0003 and
  ADR-0004 say the opposite for a major-only identifier, and a cache that
  refuses what its authority accepts is a bug in the cache. Three admission
  grounds: schema-proved (the registry's own check), data-backed (this gear
  holds the rows, so it can ask whether they all still validate), and migrated
  (a closed set of rename / default / drop steps). This narrows a normative
  MUST, which is why it is an ADR and not a note.
- **The ingest worked example is now the wire format.** It sketched
  `materialize_phantoms`, `type`, a scalar `graph_revision` and nested count
  objects; the gear ships `create_phantoms`, `type_id`, a `revision` pair and a
  flat `counts`. A producer copying the old example would not have got a
  request the gear accepts.
- **The `GraphStoreV1` listing is the shipped trait**, method by method: an
  external store implementation is written against it. Registration takes
  options, ingest takes an embedding plan, search takes the query vector, and
  five methods the first draft did not have are in it.
- **The error model says which reasons are actually on the wire.** The
  platform's builders attach a reason only to the categories that carry a
  violation; for seven others the category is the whole of what a client gets.
  The table said otherwise, and "clients never parse `detail`" was advice a
  client could not follow.
- **The readiness matrix's aggregate rule contradicted one of its own rows.**
  Resolved in favour of the row: an embedding-space mismatch blocks the vector
  arms and the gear stays ready, because taking the graph out of service for it
  would be worse than the fault.
- **The capacity table now says which rows are enforced**, where the hard
  ranges differ and why, and which belong to deferred features.
- Smaller: the property-graph migration is conditional on the server major;
  `node.version` and `edge.discriminator` were missing from the schema;
  scope replacement decides by the edge's family, not by a
  `payload.provenance.origin` member that no schema has.

`cfs validate` passes for the set: 237 artifacts, 0 errors.

## What is here

| Crate | What it is |
| --- | --- |
| `cf-gears-graph-storage` | the gear |
| `cf-gears-graph-storage-sdk` | `GraphStorageClientV1` for in-process consumers, and the three plugin contracts |
| `cf-gears-graph-storage-onnx-embedding-plugin` | a MiniLM-class model in the gear's process (off by default) |
| `cf-gears-graph-storage-remote-embedding-plugin` | an OpenAI-compatible `/embeddings` endpoint (off by default) |

A typed, multi-tenant knowledge graph over PostgreSQL: runtime GTS ontology
registration with full-chain payload validation, idempotent atomic bulk
ingest, reference and owned nodes, static and analysis edges with
provenance-preserving scope replacement, phantom materialization, lexical +
vector + hybrid retrieval with reciprocal rank fusion, depth-bounded traversal
over SQL/PGQ with a portable fallback, tabular projection over the platform's
OData binding, soft delete, and an audit envelope on every element.

The whole data plane is behind `GraphStoreV1`, and the conformance suite runs
every case against both the built-in PostgreSQL store and an in-memory fake.
That is not symmetry for its own sake: several of the defects fixed in this
branch were found by one implementation disagreeing with the other.

## What a reviewer should know before reading the code

**Three asks of the platform, all reached by building rather than by
guessing.**

1. **No row-locking surface.** The secure ORM offers no `SELECT … FOR UPDATE`
   and no `lock_exclusive`, so a gear cannot hold a row lock. The scope fence
   is written as `ON CONFLICT DO UPDATE SET generation = GREATEST(...)`, which
   *is* the lock — the only one available. The next gear needing a fence will
   write read-decide-write and not notice it is wrong.
2. **No caller-held transaction.** Every transaction API owns its closure, so
   the built-in store cannot honour "one snapshot across every arm of one
   read" and declares the capability absent. The in-memory store honours it,
   so the obligation has a passing implementation and the asymmetry is visible
   rather than assumed.
3. **`$filter` cannot reach an expression.** `FilterField::FIELDS` is a
   compile-time constant and `map_field` returns a `Column`, so payload-path
   filtering renders its own expressions inside the platform's parser, options
   and cursor. Two additive platform changes would collapse that back to one
   mapping function. A third would let a gear run `CREATE INDEX` for a
   declared path; without it equality is indexed and range and order are not.

**One test-infrastructure ask.** No published image carries PostgreSQL 19 *and*
pgvector, so this PR builds its own from a pinned pgvector on the official
base (`gears/graph-storage/docker/pg19-pgvector.Dockerfile`) and CI builds it
before the lane. It belongs in `test-containers`; until it lands, every
consumer of the lane runs one `docker build`.

**Deliberate deviations, each argued where it bites.** A denied resource is
indistinguishable from a nonexistent one everywhere except source-namespace
ownership, where the caller named a namespace whose owner is a fact about the
tenant rather than about them. An edge is returned only when both endpoints
are visible — an edge is a statement about two nodes. A tombstoned node key is
not reusable, while a re-asserted edge revives, because an edge key is derived
from its endpoints rather than held by a consumer.

**What is deferred**, with the API and schema leaving room: content chunking
and heavy-content offload, labels, change events, the admission layer beyond
per-request bounds, tenant offboarding, the analytics topology grant and
metric annotation, the index-activation lifecycle, the re-embedding lifecycle,
observability counters, and the retained type-revision history. The gear's
README carries the full list, including the places where what is built is
narrower than what is written.

## Evidence

| Lane | Cases |
| --- | --- |
| unit | 87 |
| conformance, in-memory store | 57 |
| conformance, PostgreSQL 19 + pgvector | 66 |
| domain service | 15 |
| REST, through the gear's own router | 8 |
| embedding provider contract | 2 |
| remote provider, against a mock endpoint | 7 |
| ONNX provider, against a real model | 8 |

`cargo llvm-cov` over the gear and its SDK with the PostgreSQL lane running,
enforced in CI at the 85% `nfr-code-coverage` asks for. Without that lane the
figure is ~55%: the built-in store is the half of the gear only a real server
exercises, which is why measuring and running are one job.

The four § 6.1 retrieval scenarios, timed on the seeded graph the criterion
names (100 000 nodes, 500 000 edges), release build, developer hardware:

| scenario | measured p95 | budget |
| --- | --- | --- |
| hybrid narrowing | 41 ms | 500 ms |
| criteria table | 4 ms | — |
| depth-3 typed traversal | 511 ms | 1 s |
| depth-3 UI neighborhood | 396 ms | 1 s |
| ingest 10k nodes + 20k edges | 19.7 s | 60 s |

Developer hardware under WSL2, not the reference profile: treat them as a
floor on headroom rather than a certified result. Two things the run showed
that no criterion asks about are recorded with them: edge ingest degrades from
~1 000 rows/s to under 100 as the edge table approaches half a million rows,
and a depth-3 traversal spends most of its budget on the third hop.

## Running it

```sh
make test-graph-storage          # everything that needs no database
docker build -f gears/graph-storage/docker/pg19-pgvector.Dockerfile \
  -t pg19-pgvector:latest gears/graph-storage/docker
GEARS_TEST_PG_GRAPH_IMAGE=pg19-pgvector:latest make test-graph-storage-pg
cargo run -p cf-gears-example-server --features graph-storage
```

## Notes for the merge

- Four crates, `publish` enabled; release order is sdk → gear → plugins.
- The example server registers the gear behind a `graph-storage` feature,
  alongside the gears already there.
- The gear has run continuously in an external assembly (a Studio knowledge
  graph: repository import, a domain-model ontology of 1 254 types, 531 251
  nodes and 637 975 edges) through this whole branch, which is where several
  of the corrections here came from — including the last one in it: the type
  catalogue handed out a continuation cursor that nothing would accept, which
  the suite could not see because no test had ever asked for a second page.
