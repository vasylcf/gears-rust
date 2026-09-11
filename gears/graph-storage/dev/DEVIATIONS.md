# Graph Storage v2 — deviations from the documentation

Every entry records a place where the implementation and the documentation
(PR #4523: `docs/PRD.md`, `docs/DESIGN.md`, `docs/ADR/*`) disagree, in either
direction. The list is walked after the implementation lands; each entry ends
in one of three outcomes: the implementation changes, the documentation
changes (a gap or an error found by building), or the deferral is recorded as
an accepted scope cut.

Entry format:

- **Doc:** section (file + heading) and what it requires.
- **Implementation:** what was built instead.
- **Why:** the reason for the divergence.
- **Proposal:** fix the code / fix the doc / accept the cut.

The vector-search work has its own companion report,
[`VECTOR-SEARCH.md`](./VECTOR-SEARCH.md): what was chosen and why, the
difficulties it hit, and what a live PostgreSQL 19 stand proved about it.

Categories: `[deferred]` conscious scope cut for this iteration,
`[doc-gap]` the documentation is silent or wrong, `[platform-gap]` the
platform lacks an API the documentation assumes, `[impl-gap]` the
implementation ships something the documentation does not sanction — the
category D-019 needed, since a mode the docs do not describe is neither a
silent document nor an agreed cut.

Four further forms appear in the headings, and they are outcomes rather than
new categories — an entry keeps its history instead of being rewritten or
deleted when the thing it records is dealt with:

- `[impl]` the implementation made a choice the documentation does not
  describe and does not contradict — a decision worth recording, not a gap.
- `[closed]`, `[impl-gap, fixed]` the divergence the entry opened is gone. The
  entry stays because *how it was found* is the part worth reading; the fix is
  named in it.
- `[was … now built]`, `[was deferred as D-1xx, now built]` a scope cut or a
  gap that has since been implemented, carrying the original number so a
  reader coming from the deferred list lands in the right place.
- `[doc-gap, superseded by D-0xx]` the finding stands and the conclusion drawn
  from it was overturned by a later entry. The superseding entry is named in
  the heading and the reasoning is left in place, because a conclusion that
  was wrong is evidence about the method that produced it.

**Every entry below has been checked against a running system**, not only
against the source. Where a claim was testable it was tested — on a live
PostgreSQL 19 stand, through the gear's own REST surface, or by a regression
test that fails when the behaviour it describes is reverted. Four entries said
more than the evidence supported and are corrected; the method that produced
them was writing the conclusion before running the experiment, so each entry
now records how it was verified. A second verification pass (2026-09-11) walked
the register again against the code rather than against itself and corrected
seven more claims that had gone stale as the implementation caught up — each
marked "Corrected after checking" or "Found later" in place, in the entry it
belongs to, rather than by deleting what had been said.

**Status.** Every `[doc-gap]` below **up to D-016** is folded into PR #4523
(commit `facd5da29` on `feature/graph-storage-prd-adr`), marked in the
documents as "Found while building the prototype" so a reader can tell a
decision taken up front from one the implementation forced. **Of D-017
through D-024, four now are and three are not.** D-020, D-021, D-022 and D-023
have been folded in since this paragraph was first written: the audit columns
are typed `UUID / TEXT` in both table listings (DESIGN § 3.7), PRD
`fr-tabular-projection` now says the revision rides on each row's envelope
rather than in the cursor, `GET /edges/{edge_key}` is in the PRD amendment and
in DESIGN's § 3.3 REST table, and `served_by` is in DESIGN's `ExpandResponse`
listing beside the sentence that explains why a fallback has to say so.
**D-017, D-018 and D-024 are not**: the first two came out of building vector
search after that sweep, and D-024 records a repetition that is deliberate and
still unstated in DESIGN. Checked by grepping the published documents for each
of the seven, rather than by trusting the entries' own "folded into the docs"
lines. Doc edits are agreed before they are published. The `[platform-gap]` entries are
recorded in the documents too — at the place where they bite — but they close
only with platform work, so they stay open here. The `[deferred]` entries are
accepted scope cuts and need no documentation change.

---

## D-001 [doc-gap] The SQL/PGQ ADR is `proposed`, the implementation treats it as binding

*(Written when that ADR was numbered 0006. It has been `docs/ADR/0005-…` since
the ADR set was normalized, and 0006 is now the type-evolution ADR, so the
numbers below were corrected in place.)*

- **Doc:** `docs/ADR/0005-cpt-cf-graph-storage-adr-sqlpgq-access.md`, `status: proposed` — the only non-accepted ADR; DESIGN already treats its consequences as normative.
- **Implementation:** builds on the platform layer ADR-0005's ask #3 requested (`toolkit-sea-orm-pgq` + `toolkit_db::secure::pgq`, PR #4639), i.e. treats the ADR as accepted.
- **Why:** the platform answer exists; waiting for the status flip blocks everything downstream.
- **Proposal:** flip ADR-0005 to `accepted` in PR #4523 once #4639 merges, and rewrite its "development stand exception" paragraphs — the raw-SQL exception is gone.
- **Folded into the docs:** ADR-0005 now records that its raw-SQL exception is spent and what using the platform layer changed. The `proposed` status stays: flipping it is the architect's call, not the implementer's.
- **How it was checked:** **Verified** by inspection: no `Expr::cust` and no `GRAPH_TABLE` string anywhere in the **traversal engine** (`src/infra/engine.rs`) — the pattern is built entirely by the platform builder, so the raw-SQL exception this ADR grants for *pattern access* is genuinely unused.
- **Corrected after checking.** This line used to say "anywhere in the engine", which reads as a claim about the whole gear and is not true of it. `Expr::cust` / `Expr::cust_with_values` appears in four store modules: `src/infra/store/search.rs` (the `websearch_to_tsquery` predicate, the rank expression and the cosine distance), `src/infra/store/projection.rs` (payload extraction and the `::numeric` / `::timestamptz` casts), `src/infra/store/scope.rs` (the scope-attribute extraction) and `src/infra/store/ingest.rs` (the fence's `GREATEST` upsert). Every one of them is a constant SQL fragment with the caller's data bound as a value — never interpolated — which is the rule D-010, D-030 and D-035 each record for their own fragment. None of them is a `GRAPH_TABLE` pattern, which is what the ADR's exception is about, so the conclusion above is unaffected and only its scope needed narrowing.

## D-002 [doc-gap] PostgreSQL 18+ reports `ON DELETE RESTRICT` as SQLSTATE 23001

- **Doc:** the schema section mandates `ON DELETE RESTRICT` endpoint FKs and names no SQLSTATE at all — silent rather than wrong, which the first draft of this entry got backwards.
- **Implementation:** the DB error classifier treats **both** `23503` and `23001` (`restrict_violation`) as FK violations.
- **Why:** measured while landing #4639: PostgreSQL 18 changed the SQLSTATE for RESTRICT refusals (pg16/17 → `23503`, pg18+ → `23001`; `NO ACTION` still `23503`). On PG19 every RESTRICT refusal arrives as `23001`.
- **Proposal:** add the dual-SQLSTATE note where the RESTRICT keys are specified — any gear on PG18+ with them needs it. **Done:** DESIGN § 3.7, beside the schema that declares those keys (store-specific, so not in the store-agnostic error model).
- **How it was checked:** **Verified on a live PostgreSQL 19** with `VERBOSITY verbose`, on two plain tables: `ON DELETE RESTRICT` reports `23001` (`violates RESTRICT setting of foreign key constraint`), `ON DELETE NO ACTION` on the same server reports `23503`. Exactly as recorded.

## D-003 [platform-gap] The graph test lane has PostgreSQL 19 but no pgvector

- **Doc:** ADR-0001 point 2 makes "PostgreSQL 16 or later **with pgvector**" the baseline, and PG19 the probed capability for SQL/PGQ. Both are needed at once to exercise this gear.
- **Implementation:** `libs/test-containers`'s `postgres_graph()` pins a stock `19beta3-alpine`, which has no pgvector, so `CREATE EXTENSION vector` fails and the whole schema migration cannot run. The gear's PG19 lane therefore reads `GEARS_TEST_PG_GRAPH_IMAGE` for an image carrying both, skips with a named reason when neither is available, and fails loudly when `GEARS_TEST_PG_GRAPH_REQUIRED` is set.
- **Why:** no published image carries PG19 *and* pgvector yet — pgvector gained PG19 support upstream in 2026-07 and studio-web builds its own (CNPG operand + pgvector from a pinned source revision).
- **Proposal:** platform ask — `test-containers` should expose a graph image with pgvector (built once and published, as studio-web's `docker/graph-postgres/Dockerfile` already does), so the lane needs no per-developer environment variable. Until then, DESIGN's testing section should say the two capabilities do not come in one pinned image.
- **Folded into the docs:** ADR-0001 Consequences records that the PG16/PG19 matrix has no off-the-shelf PG19 half. Stays open: it closes when an image with both is published.
- **Found later:** studio-web's image moved to the CloudNativePG operand base, which has no `docker-entrypoint.sh`, so `test-containers` (which passes `-c …` server arguments) cannot start it at all — the lane died with `exec: "-c": executable file not found`. [`dev/pg19-pgvector-test.Dockerfile`](./pg19-pgvector-test.Dockerfile) builds the missing image on the official `postgres:19beta3` base with the same pgvector pin; with `GEARS_TEST_PG_GRAPH_IMAGE=pg19-pgvector-test:latest` the whole lane ran green (26 cases, 2026-09-04). It belongs in `test-containers` so CI runs it without a per-developer build.
- **How it was checked:** **Verified** by running the gear's migration against the platform-pinned image: `CREATE EXTENSION IF NOT EXISTS vector` fails with `0A000 extension "vector" is not available`, and the schema is not created at all.

## D-004 [platform-gap] Readiness cannot name the server major

- **Doc:** DESIGN § Readiness Matrix and ADR-0001 point 2 describe SQL/PGQ as *probed*: readiness reports it unavailable on an older server, and an operator who explicitly configures it there gets a failure **naming the required major**.
- **Implementation:** the gear probes by *attempting* — at startup it runs the same pattern every hop uses, under a scope matching no rows, and takes the outcome as the answer. No catalog access is needed, so the sealed runner is not in the way.
- **Why this entry shrank.** Its first draft said "a gear cannot probe the server major" and concluded the runtime must inherit the migration's decision. The premise is true and the conclusion was wrong: what the hop depends on is whether a pattern executes, not which major answers it. Worse, the implementation matched the wrong conclusion — it assumed the capability, and a missing property graph was classified as an internal error, so **every traversal on the PostgreSQL 16 baseline answered 500** with the data reachable by the other backend the whole time. Found by dropping the property graph on the live stand. Fixed in `d9675f936`, covered by `traversal_answers_on_a_server_without_the_property_graph`, which fails with `relation "kb" does not exist` if either half of the fix is reverted.
- **What remains:** the attempt reports that the pattern did not run, never *why*. Naming the required major needs a narrow read-only capability surface (`server_version()`, `has_extension(..)`). That is a **diagnostics** gap, not a correctness one — the gear serves the right answers without it.
- **Proposal:** platform ask, filed as a diagnostics improvement rather than a blocker.
- **Folded into the docs:** DESIGN § 2.2, the readiness matrix row and ADR-0001 point 2 now say the gear probes by attempting, and separate the reporting gap from the correctness one (`3e88533f1`).

## D-005 [doc-gap, superseded by D-021] The platform page envelope has no revision slot

- **Doc:** PRD `fr-read-consistency` and DESIGN § Read Consistency Contract: every compound read reports the observed `(source_epoch, graph_revision)`, and continuation tokens are "the platform `CursorV1` **extended with the observed graph revision** — not a second token format".
- **Implementation:** the projection returns `toolkit_odata::Page<T>`, which carries `items` and `page_info { next_cursor, prev_cursor, limit }` and nothing else; `CursorV1` has no revision field a gear can populate. So the tabular projection is the one read path that does **not** report the revision. Search, traversal, node read and ingest all do.
- **Why:** using the platform binding is itself mandated (PRD `fr-tabular-projection`, and the DE0802/DE0803 lints enforce it), so a gear-local page envelope would violate a different rule.
- **Proposal:** platform ask — a revision (or opaque snapshot-identity) slot on `CursorV1`/`PageInfo`. Until then DESIGN should say which surfaces carry the revision and which cannot.
- **Folded into the docs:** PRD `fr-tabular-projection`, DESIGN § 3.3 and the drivers table no longer claim the revision travels in `CursorV1`; DESIGN § Read Consistency Contract names the surfaces that do report it and the one that cannot.
- **How it was checked:** **Verified** through the API: a projection page returns `page_info { next_cursor, prev_cursor, limit }` and no revision anywhere in the envelope, while node read, search, traversal and ingest all carry `(source_epoch, graph_revision)`.
- **Superseded by D-021.** The observation above is exactly right and the conclusion drawn from it is wrong. `CursorV1` cannot carry the revision and never will from inside a gear — but the revision does not have to travel in the *page* envelope at all. It rides on the *element*, which is this gear's own DTO, so the projection reports its observed revision on every row and there is no read path left that cannot report one. The platform ask this entry filed is withdrawn; see D-021 for what was built instead, and for why the conclusion was written before the envelope table was read.

## D-006 [doc-gap] `GraphStoreV1::end_read` is missing from the trait

- **Doc:** DESIGN § 3.3 lists `begin_read` and makes "one snapshot across every arm of one read" an obligation, but the trait has no way to *close* a snapshot.
- **Implementation:** added `end_read(ReadSnapshot)`. Without it the fake leaks a full copy of the tenant's rows per compound read, and a real store holding a transaction would leak the transaction.
- **Proposal:** add `end_read` to the trait signature in DESIGN.
- **Folded into the docs:** `end_read` is in the trait listing in DESIGN § 3.3.
- **How it was checked:** **Verified** by inspection of the trait listing in DESIGN § 3.3, which has `begin_read` and no way to close what it opens.

## D-007 [doc-gap] The built-in store cannot honour the one-snapshot obligation

- **Doc:** DESIGN § 3.3, obligation 5: two arms of one search and a hydration after it observe one revision.
- **Implementation:** the built-in PostgreSQL store declares `StoreCapabilities::snapshots = false`. A true repeatable-read snapshot needs one transaction held across several calls; `Db::transaction_ref_mapped` owns its transaction for the duration of a single closure, and the sealed runner offers no way to keep one alive across trait calls. `begin_read` returns the revision observed when the read began, and the arms are not isolated from a concurrent commit. The in-memory fake **does** honour it (copy-on-open), so the conformance case still has a passing implementation and the asymmetry is visible rather than assumed.
- **Proposal:** either a platform API for a caller-held transaction/snapshot handle, or DESIGN should mark obligation 5 as one an implementation may legitimately decline (it already has the `StoreCapabilities` mechanism for exactly that).
- **Folded into the docs:** DESIGN § 3.3 records that the built-in store declines the obligation, and why that is the capability mechanism working rather than an exception to it.
- **How it was checked:** **Verified by experiment**, not by reading. `the_built_in_store_declines_the_snapshot_obligation` opens a snapshot, commits a row from another call, and observes that row **through** the snapshot — so the absence of isolation is now an assertion rather than a comment. Inspection separately confirms the cause: every transaction API on `Db` takes a closure and `DbTx<'a>` is bound to its lifetime, so no caller can hold one across trait calls.

## D-008 [doc-gap] The attribute base has no `family`, so two ontology rules need an exception

- **Doc:** DESIGN § 3.1 authoring rules: `family` is required with no default, which is "the enforcement that stops derivation straight from a base"; and every concrete type resolves a `family`.
- **Implementation:** both rules are enforced for node and edge types only. The attribute base declares no `family` trait at all (attributes are payload fragments, not storable rows), and `provenance` derives from it directly and is concrete.
- **Proposal:** DESIGN should say the two rules are node/edge rules, or the attribute base should declare a family of its own.
- **Folded into the docs:** DESIGN § 3.1 scopes the family rules to node and edge types and says attributes have none.
- **How it was checked:** **Verified** through the API: an attribute type deriving straight from the attribute base registers successfully, where the equivalent node type deriving straight from the node base is refused. The family rules really are node/edge rules.

## D-009 [doc-gap] The prose and the table disagreed about which family types are abstract

- **Doc:** DESIGN § 3.1 prose said "three abstract bases and six **concrete** family types"; the per-family table two pages later marks four of the six abstract, `phantom_node` final and `provenance` concrete. The table was right and the prose was wrong — the opposite way round from what the first draft of this entry claimed.
- **Implementation:** four abstract families, plus two concrete by design: `phantom_node` (the gear instantiates it for an unresolved endpoint; nothing derives from it) and `provenance` (embedded in analysis-edge payloads). The test that walks the base ontology asserts exactly this split.
- **Proposal:** make the prose agree with the table. **Done:** DESIGN § 3.1, which now names which four are abstract and why the other two are not.

## D-010 [doc-gap] `websearch_to_tsquery` takes a `regconfig`, which cannot be a bound parameter

- **Doc:** DESIGN § 3.7 requires the lexical index and the query predicate to be built on the same expression, and the gear's rules forbid caller data reaching SQL text.
- **Implementation:** the text-search configuration name is inlined into the SQL (`websearch_to_tsquery('simple', $1)`), and only the caller's query text is bound. Binding the configuration as `$1` fails at runtime: `function websearch_to_tsquery(text, text) does not exist`.
- **Why:** the configuration is a compile-time constant shared with the index migration, not caller data — so inlining it is not an injection surface, and it keeps predicate and index on one expression.
- **Proposal:** note it in DESIGN beside the FTS-configuration rule; the next gear to build a lexical arm will hit it.
- **Folded into the docs:** DESIGN § 3.7, beside the schema whose index expression it has to match.
- **How it was checked:** **Verified on a live PostgreSQL 19**: `PREPARE … websearch_to_tsquery($1, $2)` fails with `function websearch_to_tsquery(text, text) does not exist`, while the inlined-configuration form prepares and executes.

## D-011 [doc-gap] `$filter` is not the right binding for the type-catalog pattern

- **Doc:** DESIGN § 3.3: "type listing binds `$filter` and `$top`", where `$filter` carries the GTS identifier pattern.
- **Implementation:** the type list takes `pattern` and `limit` as plain query parameters. Binding a GTS pattern as `$filter` is refused by the platform lints (DE0802/DE0803) because `$filter` means an OData filter expression over declared columns — which a GTS pattern is not — and the OData extractor would try to parse it as one.
- **Proposal:** correct DESIGN: the type catalog is not an OData collection; only the node projection is.
- **Folded into the docs:** DESIGN § 3.3 now says the type catalog is not an OData collection and takes plain parameters.
- **How it was checked:** **Verified** through the API, and it found a defect rather than only a mismatch: `$filter` sent to the type catalog was **silently ignored** — the same 12 types came back with and without it. Fixed in `05695faa0`; the catalog now refuses an unrecognised parameter and names the three it takes.

## D-012 [doc-gap] A producer type cannot be free-form, and its ids change shape

- **Doc:** DESIGN § 3.1 says a producer derives from a family, never from a base directly, and that `family` is required with no default. It does not say what that means for a producer that already writes types.
- **Implementation:** every consumer type id gains its family prefix — `gts.cf.studio.kg.file.v1~` becomes `gts.cf.core.graph.node.v1~cf.core.graph.owned_node.v1~cf.studio.kg.file.v1~` — and each schema grows an `allOf` `$ref` to its family. The v1 gear accepted free-form types and interned them by name, so this is a breaking change for every existing producer, and it is invisible until registration fails.
- **Why:** without a chain there is nothing to validate an instance against, which is the whole point of the base ontology.
- **Proposal:** DESIGN should carry a short migration note for producers coming from a free-form registry: the id changes, the schema needs `allOf`, and the searchable paths move from a producer-supplied `search_text` to a `full_text_search` trait.
- **Folded into the docs:** DESIGN § 3.1 carries a migration note: identifier, schema and search text all change at once.
- **How it was checked:** **Verified** through the API: a free-form type and a type deriving straight from the node base are both refused, each naming what is wrong.
- **Corrected after checking.** The example above carried a `gts.cf.core.graph_storage.node.v1~cf.core.graph_storage.owned_node.v1~…` prefix, which exists nowhere: the family segment is `cf.core.graph`, not `cf.core.graph_storage`. Checked against the nine schema files the gear publishes (`graph-storage/schemas/`), which are named for the identifiers they declare — the node base is `gts.cf.core.graph.node.v1~` and the owned family `gts.cf.core.graph.node.v1~cf.core.graph.owned_node.v1~`. The shape of the breaking change the entry describes is unaffected; a producer copying the old example would have registered against an ancestor that does not exist.

## D-013 [doc-gap] The base ontology has no stated publication moment

- **Doc:** DESIGN § Base Ontology Publication says the schemas are published, not when or by whom.
- **Implementation:** published per tenant on first type registration, prepended to the caller's batch. Not at boot: a tenant that never touches the graph gets no rows, and a tenant created later still finds its ancestors.
- **Why:** without it the very first registration fails on an ancestor nobody registered — the failure a producer sees, not the gear.
- **Proposal:** state the moment in DESIGN; the conformance suite now supplies only producer types, so the behaviour is pinned either way.
- **Folded into the docs:** DESIGN § Base Ontology Publication now says publication happens twice, in two registries, and the gear's own copy is per tenant on first registration.
- **How it was checked:** **Verified** through the API: registering two producer types returns 11 records — the two plus the nine base schemas the gear published for that tenant.
- **Found later, and larger than the entry it sits under: publication is also the only moment.** The base ontology has no update path. `with_base_ontology` prepends a base schema only when the type is absent (`if store.get_type(...).is_ok() { continue; }`), and registration converges only on a byte-identical schema — anything else under the same identifier is a conflict. So a base schema edited in the repository reaches a database that has already published it *never*: not at boot, which skips it, and not by re-registration, which refuses it. **Verified** on the stand: posting the stored base node schema back unchanged returns `200`; posting it with a single `description` string changed returns `409 CAS_CONFLICT`, "already registered with a different schema".
- **Why it matters beyond the text:** the schema is served to clients (`/types` returns `type_schema`), so the same type identifier can carry different documentation on two deployments — a fresh one and one provisioned earlier — with nothing to reconcile them. Today that is only descriptive drift, because nothing in the trait resolution reads a `description`. It stops being only descriptive the first time a base schema needs a real correction: a trait shape, an enum value, a constraint. There is no version, no migration and no override for base types, so that correction has no delivery mechanism at all.
- **Proposal:** decide the base-ontology upgrade path before the ontology-registration API freezes — a version on base types, or an explicit ontology migration that is allowed to replace them. Until it exists, treat every base schema edit as unreleasable on existing deployments, editorial ones included.

## D-014 [doc-gap] An undirected walk meets each edge twice

- **Doc:** `fr-graph-traversal` says edges are treated as undirected for reachability, and DESIGN's `ExpandResponse` carries `edges`. Neither says whether an edge reachable from both endpoints is reported once or twice.
- **Implementation:** deduplicated across hops. A two-hop walk expands the seed, reaches the neighbour, and then expanding *that* meets the very edge that led there — once as outgoing, once as incoming. Reported twice, a caller drawing or counting the result is wrong.
- **Proposal:** say it in DESIGN beside the one-hop primitive; it is not obvious from the trait.
- **Folded into the docs:** DESIGN § 3.3, beside `ExpandResponse`.
- **How it was checked:** **Verified** through the API: before the fix a two-hop walk of `f-1 → f-2 → f-3` returned `f-1 → f-2` twice; after it, once.

## D-015 [platform-gap] A local `[patch]` and the container image build are mutually exclusive

- **Doc:** the quickstart path is `docker compose up --build`.
- **Implementation:** studio-web reaches the SQL/PGQ layer through a `[patch]` block pointing at the sibling `gears-rust` checkout, which is outside the image build context — so the backend image cannot be built while the patch is in place. The stand runs the binary natively against the compose PostgreSQL instead (`config/local-stand.yaml`).
- **Why:** `toolkit-sea-orm-pgq` is unpublished and lives on a branch.
- **Proposal:** none needed — it resolves itself when PR #4639 merges and the git dependencies move back to `main`. Recorded so the next person does not spend the afternoon finding it.

## D-016 [deferred] Two read-surface defects the sweep found

Neither is a divergence from the specification — the specification was right
and the implementation was not — but both were invisible until the entries
around them were checked against a running system, so they are recorded where
that check happened.

- **Every `OData` failure was reported as a breached bound.** An unknown filter
  field answered `out_of_range` / `LIMIT_EXCEEDED` with the field `limit`,
  telling a caller to reduce a value they never sent. Now classified by what it
  is, and the rejection names the fields the projection accepts, which
  `fr-tabular-projection` asks for.
- **The type catalog ignored parameters it did not recognise.** `$filter` sent
  there changed nothing and said nothing — the failure mode the projection's
  binding exists to prevent, on the surface next to it.

Both fixed in `05695faa0`.

---

## D-017 [doc-gap] The four vector states are named, not encoded
`fr-embedding-pipeline` fixes the states a vector may be in after an upsert — embedded and current, absent, preserved, stale — and says similarity search must consider only current ones. It does not say how a store distinguishes them, and the columns DESIGN gives (`embedding`, `embedding_epoch`, `embedding_input_hash`) admit more than one encoding.

**Chosen:** `embedding_input_hash` describes the *stored vector's* input, never the node's current text, which is what lets a later ingest tell a preserved vector from a stale one. `embedding_epoch` carries currency: the active epoch for a current vector, `NULL` for one that is absent or stale. The arm then reads `embedding_epoch = <active>`, so "only current vectors rank" is one equality rather than a rule every query has to remember, and the HNSW index is partial on the same distinction, so a stale row does not occupy a slot in the candidate set.

**Why it is worth stating:** the encoding is a store-visible contract. An external `GraphStoreV1` plugin has to arrive at the same distinctions, and the FR alone does not tell it how. The decision itself lives in `domain::embedding::decide_vector`, shared by both implementations, so the two cannot drift — the lesson of the endpoint-constraint entry above.

**Proposal:** DESIGN's `node`/`chunk` table notes should say what `embedding_epoch = NULL` means beside a non-NULL `embedding`, since that is the only state the column names cannot be read off. Still open: unlike D-020 through D-023, this one has not been folded in — the two table rows say "staleness detection" and leave the encoding to the reader.

**How it was checked:** by reading the two places that have to agree. `m0003_embedding_space.rs` drops the index `m0001` created and rebuilds it `WHERE deleted_at IS NULL AND embedding IS NOT NULL AND embedding_epoch IS NOT NULL`, and the arm's own predicate is `embedding_epoch = <active>` (`src/infra/store/search.rs:172`) — so the index's partiality and the query's equality express the same distinction, and a stale row is outside both. Worth recording because a verification pass over this register reported the predicate as `deleted_at IS NULL AND embedding IS NOT NULL`: that is `m0001`'s index and `m0003`'s own `down()`, not the migrated state, and on a database that has run every migration the sentence above holds as written.

## D-018 [doc-gap] `embedding_space` is deployment-wide, and every runtime read is scoped
DESIGN's `embedding_space` table has `epoch` as its primary key and no tenant column: it is deployment-wide, correctly. But every runtime read in this gear goes through the secure ORM, which needs a scopable entity, and there is no unscoped read API — by design.

**Implementation:** the table carries a `tenant_id` holding the nil UUID, and boot reads it under a nil-tenant scope. This is the gear's existing device rather than a new one: `graph_meta` already holds deployment-level keys the same way, and `probe_pgq` already reads under a nil-tenant scope. Nothing reads the table per request — the active epoch is resolved once at boot and carried in memory — so the column costs one write at first boot and nothing thereafter.

**Proposal:** either DESIGN notes the column for stores built on a scope-enforcing ORM, or the platform offers an explicit deployment-scope read. The second is the better shape; the first is what the prototype could do.

## D-019 [impl-gap] The gear's default embedding provider carries no semantics
A deployment that does not configure `graph-storage.embedding_provider = onnx` gets the deterministic fake, and vector search then answers with rankings that mean nothing — reproducible, well-formed, and semantically arbitrary. Boot says so at `warn` level and that is all.

ADR-0004 has no such mode: its three providers are ONNX, remote and "a deterministic fake for CI". Shipping the CI fake as the *runtime default* is this prototype's choice, made so the write path, the epoch bookkeeping and the four vector states could be exercised before a deployment has model artifacts. It is defensible for a prototype and wrong for a release: the failure it produces is a quiet quality loss, which is the exact failure mode ADR-0004 is written to prevent.

*(Renumbered in place: this entry cited ADR-0005 until a verification pass caught it. The embedding-provider decision is `docs/ADR/0004-cpt-cf-graph-storage-adr-embedding-provider.md`; ADR-0005 is SQL/PGQ access, which has nothing to say about providers.)*

**Proposal:** before release, either make `onnx` the compiled-in default with no fallback, or make an unconfigured provider a boot failure. A warning in a log is not a guard.

## D-020 [doc-gap] The envelope's subject id is a `Uuid`, and DESIGN types it `TEXT`

DESIGN § 3.7 gives the audit columns as `created_by_subject_id / created_by_subject_type | TEXT`. The vocabulary that column is pointing at is `SecurityContext`, whose `subject_id()` returns a **`Uuid`** — `subject_type()` is the `&str` half, a GTS type identifier.

**Implementation:** `UUID` for the id, `TEXT` for the type (migration `m0004`). Writing a `Uuid` into `TEXT` widens the domain for nothing, costs 20 bytes a row against 16, and makes an equality join against any other subject column a text comparison.

**Proposal:** fix the doc — one word in two table rows. This is not a design disagreement, it is a table sketch written before the platform type was looked up.

**How it was checked:** **Verified on the live stand.** `\d node` on the PostgreSQL 19 instance shows `created_by_subject_id | uuid | not null`, and a node read back over REST carries `"subject_id": "00000000-0000-0000-0000-00000000a001"` with `"subject_type": "gts.cf.core.security.subject_user.v1~"` — both halves populated from the request's own security context.

## D-021 [doc-gap] `graph_revision` on the element is what lets the projection report one at all

PRD § `fr-tabular-projection` carries a "Found while building the prototype" note: the continuation token cannot be `CursorV1` *extended with the observed graph revision*, because `CursorV1` has no revision member and the platform page envelope carries only items and cursors. It concludes that tabular projection is **the one read path that does not report `(source_epoch, graph_revision)`**, and that closing it needs a platform slot.

**It does not.** DESIGN's envelope table already lists `graph_revision` as an envelope member — "the revision the read observed" — and the envelope rides on the *element*, not on the response. `toolkit_odata::Page` constrains the wrapper; it does not constrain the items, which are this gear's own DTO. So the projection reports its observed revision on every row, and the gap closes with no platform change.

**Implementation:** `ElementEnvelope.graph_revision`, resolved from the call's compound-read snapshot when it has one and from `graph_meta` otherwise. Per element rather than per response, which duplicates one value across a page — the cost of the platform wrapper having no place for it.

**Proposal:** replace the PRD note. The finding it records is real (the cursor cannot carry it); the conclusion drawn from it is not.

**How it was checked:** **Verified on the live stand.** `GET /nodes?limit=2` returns each row with `"graph_revision": {"source_epoch": 1, "revision": 5}`, and the conformance obligation `a_projection_row_carries_the_envelope` asserts it against both store implementations.

## D-022 [closed] No read surface returned an edge, so half of `fr-audit-envelope` was unexercised

`fr-audit-envelope` says **every node and edge** returned by any read surface must carry the envelope. The prototype had no surface that returned an edge as an element: `DELETE /edges/{edge_key}` existed, `GET` did not, and an edge appeared in a response only as a topology reference — `EdgeRef` in a traversal, `AdjacencyEntry` in a node read — which is a key, a type and two endpoints by design.

The edge table carried the full envelope and every write populated it (migration `m0004` adds `updated_at` and the three subject pairs), so the data was there and correct. Nothing read it back — and a requirement that cannot be observed is a requirement nothing tests: the columns would drift the first time an ingest path was added that forgot one.

**Closed by implementing the read the FR implies.** `GraphStoreV1::get_edge` on both stores, `GET /api/graph-storage/v1/edges/{edge_key}` returning `GraphEdgeDto` (key, type, endpoints, discriminator, payload, envelope). The key is the one the topology references already carry, so an adjacency entry or a traversal reference is directly addressable — `an_edge_read_carries_the_envelope` reaches the edge exactly that way rather than re-deriving the hash, which is also what asserts the two surfaces agree on how an edge is named.

Scoping follows the induced authorized subgraph: the edge is returned only if **both** endpoints are visible under the caller's scope. An edge is a statement about two nodes, so returning it while an endpoint is hidden would leak the connectivity the node read refuses to.

**Proposal:** PRD's `fr-audit-envelope` and DESIGN § 3.3 both carry the surface now (amendment notes in place).

**How it was checked:** three conformance cases against both stores — `an_edge_read_carries_the_envelope` (creator preserved, last writer moved by a second producer re-asserting the same relationship, tombstoned and unknown keys alike absent), `an_edge_whose_endpoint_is_hidden_is_not_readable` (another tenant's key, and an endpoint tombstoned within one tenant), and `both_node_families_and_both_edge_families_round_trip`, which reads every edge of the batch back through the edge read.

## D-023 [doc-gap] "Falling back is never silent" was silent to everything except a human reading logs

DESIGN's traversal section says the pattern backend's decline is "never silent — the reason is logged either way", and the implementation did exactly that: `warn!` on `PatternOutcome::Unavailable`.

**A log line is not observable to a test.** Adapting the gear to PR #4639's anchor opt-in, the fix was removed to see what would fail, and **nothing did**: the pattern statement referenced a relation absent from its `FROM`, PostgreSQL refused it with `missing FROM-clause entry for table "node"`, every traversal was served by the two-query fallback, and the whole PG lane passed. `the_pattern_hop_walks_the_graph` walked the graph without the pattern; `both_hop_backends_return_the_same_answer` compared the fallback with itself and found perfect agreement.

**Implementation:** `ExpandResponse::served_by`, a `HopBackend` of `Pattern` or `TwoQuery`. The two tests now assert which backend answered them.

**Proposal:** DESIGN should say the backend is reported *on the answer*, not only logged. The general form is worth stating once: a fallback that returns the right answer is invisible unless the answer says which path produced it — a property this gear has now met twice, here and in D-004.

**How it was checked:** **Verified by control experiment on a live PostgreSQL 19 stand.** With `correlate_with_anchor()` removed, exactly those two tests fail on exactly those assertions and the other 22 pass; restored, all 24 pass and the stand's own log records zero fallbacks across every traversal served over REST.

## D-024 [doc-gap] The envelope's `key` repeats a node's `node_key`

DESIGN's envelope table gives `key` as an envelope member on both element kinds — "echoes the producer's id" for a node, the derived hash for an edge — while the node base type declares `node_key` as producer-authored. A node read therefore carries the same string twice, once in the body and once in the envelope.

**Kept, deliberately.** Dropping `node_key` from the body would break the round-trip the envelope contract asks for ("a document read from the API can be sent back unchanged"): ingest addresses a node by a top-level `node_key`. Dropping `key` from the envelope would make the envelope a different shape for nodes and for edges, which is the one property it is defined by.

**Proposal:** none — but DESIGN should say the repetition is intended, because the alternative reading is that one of the two is a mistake.

## D-025 [impl] The remote embedding provider is built, without the egress policy ADR-0004 puts in front of it

D-103 deferred the remote plugin with the reason that *"building the plugin without that policy would be building the part that is easy to get wrong"*. It is now built anyway (`gears/graph-storage/remote-embedding-plugin`, behind the gear's `remote` feature), and the reason is a deployment fact rather than a change of mind: the reference assembly has to run where a model in the gear's process is not affordable — a memory ceiling, no CPU budget for inference — and a deployment that cannot embed at all is the option ADR-0004 rejects first.

- **What is built:** the `OpenAI`-compatible `POST /embeddings` protocol, batched, aligned by the response `index`, width-checked per vector, L2-normalized on this side; the credential named by environment variable rather than carried in configuration; the SDK's executable provider contract passing against a mock endpoint, as ADR-0004 asks ("remote via mock server").
- **What the identity can promise, and cannot:** the space is named by *model at endpoint at width*. That is the only identity a remote endpoint offers, and it cannot see a vendor changing weights behind a stable model name. The ONNX provider's content hash can; this one relies on model governance.
- **What is still not built:** the default-deny per-tenant egress policy (vendor, endpoint, region, data classes, vectorized fields, retention terms). Until it exists, selecting `remote` means every tenant's node text and query text leaves the deployment for the one configured endpoint, and the configuration file is the only record that it does. **A prototype trade, not a release posture.**
- **Proposal:** the egress policy is a platform concern shared with the LLM gateway (OAGW already models approved upstreams per tenant); the plugin should route through it once a per-tenant upstream can be resolved for embeddings, and the direct client here becomes the no-gateway fallback.

## D-026 [impl] The base-ontology schemas moved into the crate, because `cargo package` ships nothing outside it

The nine base schemas lived under `docs/schemas/` and reached the binary through `include_str!("../../../docs/schemas/...")`. That compiles from a checkout and fails to package: a crate's archive contains only its own directory, so the first `cargo publish` — or the first consumer building the gear as a git dependency — would have found no schemas. They now live in `gears/graph-storage/graph-storage/schemas/` (with the `acme` examples beside them) and DESIGN points there. The gear registers the same bytes; only the path changed.

## D-027 [was impl-gap, now built] Unchanged nodes were re-embedded on every ingest

`EmbeddingCoordinator::plan` composed and embedded every node of a batch before `decide_vector` compared the input hash with the stored one, so the *preserved* state was decided correctly but paid for as if it were *embedded*: a byte-identical re-sync of an 824-node repository through the reference consumer took 25 s with the in-process ONNX provider and changed nothing.

**Built.** `GraphStoreV1` gains `embedding_state(keys)` — the stored input hash and the epoch the stored vector is current under, index-aligned, unknown and unauthorized keys reading alike as `None`. The domain service reads it before planning and the coordinator embeds only the inputs whose hash or epoch differs, in one provider call; the rest are planned as `skipped`, which the store's `decide_vector` resolves to *preserved* as before. Covered three ways: unit tests on the coordinator with a counting provider, a conformance case (`an_unchanged_re_ingest_embeds_nothing`) run against both stores, and the studio-web stand.

**What the read outside the transaction costs.** A node changed by a concurrent writer between the state read and the write is planned as skipped and lands *stale* (vector kept, not rankable) until the next ingest touches it — the same state `embed: false` produces on purpose. The alternative, embedding inside the transaction, would hold row locks across a provider round trip. Accepted for the prototype. **The readiness surface now exists (D-033), and it still does not answer this.** What it cannot report is a *number*: `ComponentReadiness` carries a component name, a state, the problem, what the state blocks and what is being waited on, and no counters at all, so a deployment can see that the embedding space is healthy and not how many rows are sitting stale under it. That count is the open half of this entry, and it is the one an operator would act on — stale rows are silently missing from the vector arm until something touches them.

## D-028 [impl-gap] Lexical search cannot find identifiers inside file names

`compose_search_text` joins the declared paths and the store indexes them, and a name like `README.md` or `rust-watch.Dockerfile` arrives in the index as one token. On the stand, `Dockerfile` matched and `README` and `rust` did not, although both name files.

**The configuration is not the cause, and the first draft of this entry blamed it.** The store indexes with the `simple` configuration already — one constant, `FTS_CONFIG` in `src/infra/storage/migrations.rs`, shared by the generated `tsvector` column (`to_tsvector('simple', search_text)`, `m0001`) and by every `websearch_to_tsquery` predicate, precisely so the index expression and the query cannot drift (D-010). What produces the single token is the default **parser**, which classifies `README.md` and `rust-watch.Dockerfile` as its `file` and `url` token types *before* any configuration is consulted; a text-search configuration maps token types to dictionaries and cannot make the parser split a token it has already emitted. So the remedy the first draft proposed — "the store should index a second, `simple`-configuration vector for identifiers" — would change nothing at all, and is withdrawn.

What is left is a composition change: the searchable text should also carry the punctuation-split tokens of a name, composed once in `compose_search_text` (`src/infra/store/ingest.rs:75`) so every producer inherits it rather than each remembering a `name_tokens` payload member of its own. The alternative that does work store-side is a second column parsed differently, which is a schema change and a bigger decision than this gap needs.

**How it was checked:** `FTS_CONFIG` read at its single definition and followed to both users; the token-class behaviour is the default parser's, which no configuration in the gear replaces.

## D-029 [impl] The derivation-chain ceiling is a configured policy, not a fixed three

`guidelines/GTS.md` § 9 *recommends* keeping a chain to two derivations (three segments), and the prototype had hard-coded that number in `ontology::analyze`, with DESIGN saying "a producer type is always the third segment and can never introduce a hierarchy of its own beneath it". Checked against the type system: neither the `gts` parser nor the registry limits chain length (the only hard bound is 1024 characters per identifier), and nothing in this gear depends on it — `ancestors`, trait resolution, the chain validator and pattern matching work on any length. A consumer mirroring a domain model whose hierarchy is deeper than one level under the family (`managed_object ~ document ~ requirement`) gains from the longer chain exactly what chains are for: one pattern on the intermediate (`…managed_object.v1~*`) selects the subtree in every read and in every `src_types`/`dst_types` constraint, and a trait declared once on the intermediate reaches every leaf.

**Chosen:** `ontology_max_chain_depth` (default 3, hard range 3..=16). The default keeps the platform posture; a deployment raises it deliberately. The guideline's argument — deep chains are hard to reason about and review — is about types authored in code and reviewed in gears-rust; a tenant's ontology is data registered at runtime, generated from one model, and the reasoning lives in that model. Two rules for authors of deeper chains: an intermediate type may not close `payload` (`additionalProperties: false` there rejects every descendant's members, because `allOf` branches evaluate independently — rule 3 above), and trait *values* replace along the chain rather than accumulate (JSON Merge Patch), so a leaf that restates `index` restates all of it. Covered by unit tests on both postures and a conformance case on both stores (`a_deeper_chain_registers_and_its_ancestor_admits_the_leaf`).

## D-030 [was deferred as D-104, now built] `$filter` and `$orderby` over declared payload paths

The projection now serves payload paths. Registration resolves every `index` pointer against the type's schema chain to a scalar kind (`string`, `number`, `integer`, `boolean`, `date-time`) and refuses one that lands nowhere or on an object or array — the second thing D-104 said the ADR left to the reader. The kinds are stored beside the traits (`effective_traits.index_kinds`). A projection that names a payload path takes a gear-owned rendering of the shared plan (`domain::projection`): admissibility is *every selected type declares the path with one kind* — a path some selected type lacks would silently drop that type's rows, which is a wrong answer, not a narrower one — and the store renders the plan as extraction expressions (`payload #>> '{a,b}'`, typed through `jsonb_typeof` guards) with bound literals, in the platform pager's statement shape: filter, keyset predicate, order with nulls last in either direction, `limit + 1`, `CursorV1`. Column-only projections still take the platform pager unchanged. Both stores run the same conformance cases; the built-in store additionally proves keyset paging over a payload ordering.

**What the platform gap in D-104 turned out to mean.** The two asks stand (a per-request field set; a field that maps to an expression), and this iteration does not wait for them: it renders the plan itself. That is the alternative ADR-0003 refused as "a second query dialect". It is not one — the parser is the platform's, the accepted options are the platform's, the cursor is the platform's `CursorV1`, and the only thing the gear owns is the mapping from an admitted identifier to an expression, which is exactly what ask 2 would let it hand back. When the asks land, `infra::store::projection` shrinks to that mapping.

**What remains, deliberately.**
- *Equality is indexed; range and order are not.* Equality over a declared path is written in containment form (`payload @> '{"a":{"b":"x"}}'`) and served by one static GIN (`jsonb_path_ops`, migration `m0005`). The B-tree per declared path D-104 calls for needs `CREATE INDEX` at registration, and the platform's secure ORM exposes no statement surface a gear could run DDL through — by design, and this iteration does not work around it. Range comparison and ordering therefore read the extraction expression over the rows the type index narrowed: correct, bounded by the page and the type set, and without an index of their own. This is the number to watch in the load test, and the thing that makes ask 3 to the platform a DDL surface, not only the two field-binding asks.
- *Numbers are compared as numbers; a `date-time` path is compared as text.* Exact for RFC 3339 timestamps written in one offset, approximate across offsets. A `::timestamptz` cast would be exact and would fail the statement on one malformed string, and the gear does not rely on `format` assertions for that guarantee.
- *Forward paging only.* A payload-ordered projection mints `next_cursor` and no `prev_cursor`; a backward cursor is refused with a message.
- The `index` trait's `description` in the base schema still says "backed by a JSONB index" — true again, for equality; unchanged for the reason D-013 gives.

## D-031 [impl-gap] A registered type's schema can be updated in place, on two grounds

The gear registered a type once and answered `409` to any changed schema under
the same identifier. DESIGN sanctions that ("a different schema under a
registered identifier is a conflict") and says nothing about evolution, so this
entry is `impl-gap`: what is built goes beyond what the documentation
describes, and the documentation is the thing that is wrong.

**The policy is not the gear's to choose, and the gear was diverging from it.**
types-registry ADR-0003 fixes the direction (`BACKWARD`: a candidate is
admissible when `Valid(baseline) ⊆ Valid(candidate)`), the baseline (the
entity's *current* revision, never its history) and the posture (an undecidable
check is a refusal — the registry fails closed). ADR-0004 says a major-only GTS
id names a **mutable** logical entity and that *"a content update that is
backward compatible under ADR-0003 preserves the same GTS ID"*; ADR-0005 makes
every admitted definition a retained revision. Our own `gts_type` docstring
calls the table "a cache with a foreign identity, never a second source of
truth" — and a cache that refuses what its authority accepts is a bug in the
cache.

**Nothing of the relation was written here.** `gts 0.12`'s `schema_evolution`
is the evolution check (spec sec 4.2, **OP#8**): `check_backward_diagnostics`,
`check_forward_diagnostics`, the three-valued `CompatibilityVerdict`, and
diagnostics carrying the offending **schema location** so a refusal can point
at `$.payload` instead of saying "incompatible". `GtsStore::compare_documents`
resolves both documents' `$ref`s first and also classifies every object level's
content model, which is what `is_evolvable_in_place` reads. OP#12 is schema
*derivation* and is a different relation over a different pair of schemas; the
gts docs are emphatic that the two must not be conflated.

**What was built.** `options.on_existing: reject | update` on `POST /types`
(`reject` is the default and is the previous behaviour byte for byte), a
`POST /types/compatibility` dry run that runs the identical path and writes
nothing, `revision` and `updated_at` on `gts_type` (`m0006`), and a per-type
report: state, both directional verdicts, every diagnostic with its location,
which traits moved, the type's row count, the object levels a *later* edit will
not be able to extend, and whether the change is admissible. Forward
compatibility is computed and reported, never enforced — the registry's own
posture, and useful to a producer, since an added optional property is exactly
where the two directions disagree.

**The one thing the gear owns, and it is deliberately a second ground rather
than a looser first one.** ADR-0003 decides from the schemas, because a
registry has nothing else. This gear holds the data, so when the schemas cannot
prove inclusion it can ask a different question: does every live row of the
type validate against the candidate? `options.revalidate` opts into that, the
answer is reported as `admission_basis: "data_backed"` with the number of rows
read, and it is never cached or restated as a verdict about the type — it is
true of *these rows*, and a later ingest of the old shape fails, which is the
point of the change. Bounded by `type_update_max_rows` (default 100 000):
above it the honest answer is an asynchronous migration with progress, which
does not exist. A provably compatible change reads no row at all.

**Measured before any of it was written, on the Studio domain model
(2026-09-10).** Over the 188 instantiable node types the exporter emits, "add
one optional property" comes back **`incompatible` 188 times out of 188** — and
`Unknown` never. The reason is gts sec 4.4: the payload object level is *open*,
so the previous definition already accepted any value under the new property's
name, and declaring it *narrows* the accepted set. With the payload
materialized and closed at the leaf (`additionalProperties: false`, inherited
properties restated) the same edit is **`compatible` 188 times out of 188**.
Two consequences:

- the exporter should close the payload on any type with no descendants — 181
  of the 188 are leaves — and only there, because an intermediate type that
  closes `payload` makes every descendant uninstantiable (the rule D-029
  already states);
- until it does, those edits go through the data-backed ground, which is
  exactly what it is for. This is why the dry run shipped first: the number
  above is a measurement, not a guess, and it changes the exporter rather than
  the gear.

**Rehearsed on the stand (2026-09-10), against the loaded model** — 1254
types, 531 251 nodes, 637 975 edges; `m0006` applied on boot. The four PM edits
against `…cf.studio.sdlc.requirement.v1~` (1 000 rows): widening the `status`
enum is schema-proved in 61 ms with no row read; adding an optional property
and renaming one are admitted data-backed over 1 000 rows in 230–360 ms;
making a property required is refused naming the row (*node `requirement:1`:
"owner" is a required property*). Applying the proposed exporter shape for
real: closing the payload level (29 inherited properties restated at the leaf)
is admitted data-backed in 270 ms, and the *next* edit — add an optional
property — is then `schema_proved` in 62 ms with no row read. At scale, the
250 000-row `action_run` type re-validates in 12.9 s (≈19 400 rows/s), so the
shipped 100 000 ceiling is ~5 s; above the ceiling the refusal names both the
count and the config key. Full run in
[`type-update-plan.md`](./type-update-plan.md) § The stand rehearsal.

**Two limits the rehearsal exposed, both now recorded rather than papered
over.**
- *`data_backed` means "no stored row becomes invalid", not "no query breaks".*
  Over an **open** payload the rename `priority → urgency` is admitted — no row
  becomes invalid, because an open level still accepts the `priority` the rows
  carry — while every `$filter=payload/urgency` then returns nothing. Over a
  **closed** payload the same rename is refused, naming the rows that still
  carry `priority`. The ground is exactly as strong as the shape it checks
  against, and a rename needs the migration of slice 3 either way.
- *The scan did not consult the caller's deadline.* 250 000 rows take longer
  than the 10 s interactive deadline, and only the row ceiling stood between a
  raised limit and a request outliving itself. The scan now checks the
  remaining budget between batches and answers `Deadline`: the ceiling bounds
  the work, the budget bounds the wait. It is the first store path in this gear
  to consult `StoreCtx::budget`, because it is the first whose duration scales
  with a tenant's data.

**A third ground, built 2026-09-10: the caller may move the data.**
`POST /types` takes `migrations` — at most one plan per type — as a closed set
of three steps: `rename` moves a value where the source is present, `default`
fills what is absent or null and never overwrites, `drop` removes what is
there. Reported as `admission_basis: "migrated"` with the rows scanned and the
rows actually changed. Three properties worth stating because each was a
decision:

- *The steps run in Rust and the store writes what it validated.* The plan's
  §4.3 sketched `jsonb_set` expressions; a step engine in Rust beside an
  expression chain in SQL is two implementations of one migration, and a row
  can pass one and fail the other. Every row is read anyway to validate it, so
  read-modify-write costs what we were paying regardless. One statement per
  *changed* row, which is what the row ceiling keeps honest.
- *A migration needs a schema change to migrate towards.* Offered against a
  byte-identical candidate it would be a payload-editing API wearing a type
  registration's clothes — a different feature, with a different authorization
  story — so it is refused, as is a migration naming a type the batch does not
  register (a typo would otherwise do nothing at all).
- *A migration is a write, and carries every obligation a write carries.*
  `updated_at`/`updated_by` record the migrating subject, `node.version` moves
  (a producer holding the pre-migration value would otherwise silently undo
  the migration through `expected_version`), the graph revision advances, the
  lexical text is recomposed from the new payload, and the vector epoch is
  cleared — but only for a type that composes its embedding input from the
  payload. That last clause is a divergence the conformance suite caught: the
  fake was clearing it for every type, which would re-embed a whole type for
  nothing.

**Rehearsed on the stand.** The rename that motivated all of this, against the
1 000-row `requirement` type: dry run 122 ms, applied in **791 ms**, 1 000 rows
scanned and **949 rewritten** — the 51 that never carried `priority` are not
touched, because a rename moves what is there. Before, `$filter=payload/urgency`
was a `400` (an undeclared path); after, it filters and orders, and
`requirement:1` carries under `urgency` the value it used to carry under
`priority`. Renaming back restored the stand.

**What is not built.** The retained-revision history table and
`GET /types/{id}/revisions` — **deferred by decision (2026-09-11)** rather than
unfinished: the revision counter says which definition is in force, no surface
reads the history, and the trigger for building it is a consumer that needs the
previous definition. Also open: a backfill that re-embeds what a migration
marked stale. The marking is done and the worker is not, so vector search covers
fewer rows until something touches them — and the remedy meanwhile is a plain
**re-ingest of the type**, which producers do routinely: the epoch is cleared,
so the coordinator re-embeds the row even when its text is unchanged. The count
is in the migration's own report (`rows_rewritten`, for a type that declares
`vector_search`). This is deliberately **not** scheduled as its own task: the
PRD already requires a re-embedding lifecycle for the larger case (a provider or
model change blocks the vector arm the same way, § fr-embedding-space and the
tenant-offboarding and fairness requirements all name re-embedding jobs), none
of it exists, and a backfill built for migrations alone would be one of three
inputs to a mechanism nobody has written. **The gear has no background task of
any kind, and three separate needs are queued behind the one that does not
exist:** a backfill that re-embeds what a migration marked stale; an
asynchronous migration for a type over the row ceiling; and the delegation of
the verdict to types-registry over the wire. The call site is one function (`domain::evolution`) precisely so
that last one can replace it without touching the store or the API.

**Authorization.** Registration stays `admin` on the type resource. A
re-validating update additionally requires `write` on the node resource and is
served under *that* scope: it reads the tenant's rows, and holding ontology
administration is not holding the data.

**A second door to an already-recorded gap.** ADR-0003 makes changing an
annotation a type-version change with a durable index activation lifecycle
(`requested -> building -> active`) and admits a filter only while the path's
index is `active`; `fr-index-admission` demands capacity admission before index
intent commits. Neither exists (D-104, D-105), which is why a declared path is
filterable the moment it is declared. The in-place update does not create that
gap, but it opens a second way in: a new `index` path can now appear under an
*existing* identifier, with no lifecycle and no capacity check. Named here so
that whoever builds the lifecycle gates this path too.

**Found by reading the docs against the code, and fixed.** An accepted update
now advances the tenant's graph revision, once per updated type, inside the same
transaction. The Read Consistency Contract's promise is that two reads at one
revision cannot observe different content, and an updated type changes what a
read answers — the projection admits a path it refused a moment ago, and ingest
validates against a different schema. `fr-labels` carries the same obligation
for the same reason ("attach and detach **MUST** increment the tenant's graph
revision, so two reads at one revision can never observe different labels"). A
`created` type changes no existing read and still leaves the counter alone,
which is what registration always did; the conformance case pins both halves.

**Two row ceilings, because the two passes run at different rates
(2026-09-11).** Measured on the stand: re-validation *reads* at ~19 000 rows/s
(250 000 in 12.9 s, 1.1 M in 105 s), a migration *rewrites* at ~1 900 rows/s
(10 000 in 5.1–6.3 s, three runs) — it is one statement per changed row. Under
the gateway's 30 s that is the difference between a bound and a trap: at the
shared `type_update_max_rows` of 100 000 a migration is admitted, does ~52 s of
work, and is killed. `type_migration_max_rows` (default **25 000**, ~13 s at the
measured rate) bounds the migration pass instead, and the refusal names the key
that actually applies — an operator who is told the wrong key raises the wrong
one. Verified on the stand from both sides: the 10 000-row `audit_entry` still
migrates, and the 250 000-row `action_run` is refused with *"one synchronous
pass handles at most 25000 (`type_migration_max_rows`)"*.

The measured evidence that a timed-out request rolls back rather than half
committing: the 866 000-row re-validation that returned `504` left nothing
behind — the chunked run afterwards reported all 406 types as `updated`, not
`unchanged`.

**Neither ceiling exists in the fake store**, which carries no configuration;
the conformance suite therefore covers the *grounds* on both implementations and
the *bounds* on neither. The bound selection is a unit case over the pure
function, and the numbers above come from the stand.

**The row ceiling is set by the gateway, not by the gear's own deadline.**
`api-gateway` kills any synchronous request at 30 s — `SYNC_TIMEOUT`, a
constant in the proxy rather than a setting — so that, and not
`deadline_interactive_secs`, is what a re-validating update has to fit. At the
~19 000 rows/s measured in-gear the shipped `type_update_max_rows` of 100 000 is
about 5 s, comfortably inside; 250 000 is not. Worth saying because the gear's
own deadline can be *configured* above the gateway's ceiling (its hard range
runs to 300 s), which buys nothing: raising it to 300 and posting a
866 000-row re-validation returned `504 Request exceeded 30s timeout` on the
stand. A transition over a loaded graph is therefore chunked by the caller, or
asynchronous — which is the shape D-031 already says is out of scope.

**One asymmetry left standing on purpose.** The in-process door
(`GraphStorageClientV1`) offers only the default mode: `register_types`, no
options, no dry run. Widening it means adding methods to a published trait, and
`cpt-cf-graph-storage-interface-sdk-client` states the policy — "breaking
changes introduce a new trait version". So the in-process consumer is *narrower*
than REST rather than weaker, which is safe (it cannot reach a behaviour REST
would refuse), and widening it is a `ClientV2` question rather than something to
smuggle into V1.

**Folded into the docs.** The documentation said `MUST reject`, so a deviation
entry alone would not have been an outcome. Written and published with the
implementation:
[`docs/ADR/0006-cpt-cf-graph-storage-adr-type-evolution.md`](../docs/ADR/0006-cpt-cf-graph-storage-adr-type-evolution.md)
(`accepted` 2026-09-10 by the gear owner on the conformance and stand evidence,
with the width of that agreement stated in the ADR itself, because it narrows a
normative MUST), plus amendment notes rather than silent rewrites at each
place the old rule is stated — PRD `fr-type-registration` and the § 12 risk row,
DESIGN § 2 traceability, § 3.3 (the new operation, the new option, and why
asking is a separate operation from doing), § 3.7 `gts_type` (two columns, and
what "a row per minor version" now means), § 4 Ontology Registry
responsibilities, and ADR-0003's annotation consequence. What is deliberately
**not** amended: `fr-index-admission` and the activation lifecycle, which stay
unimplemented deferrals rather than being redefined to match what is built.

## D-032 [was impl-gap, now built] The source-namespace boundary was not a boundary

**Found while planning the upstream contribution (2026-09-11), by reading
`fr-source-ownership` against the code rather than against the other entries.**
`node.source_namespace` was written `None` and `node.owner_principal` was
written `""` on every path, and nothing read either. The documentation calls a
source namespace "an enforced ownership boundary" with immutable owner
provenance and per-update re-authorization; there was no registry, no claim, no
comparison. Exactly the shape of the gaps the previous sweep did catch — parsed,
stored, unread — and it is p1.

No entry recorded it, which is the part worth remembering: the sweep walked the
PRD's acceptance criteria (§ 9) and this requirement has no criterion of its
own, so it fell between the two checks.

**Built.** `source_namespace_owner` (migration `m0007`) is the authority;
`domain/ownership.rs` reads the namespace out of a reference node's own payload
and decides claim / allow / forbid; the ingest path authorizes before either
branch writes, claims an unclaimed namespace inside the write transaction, and
writes the two node columns once on insert. `GET /source-namespaces` reads the
boundary and `POST /source-namespaces/{namespace}/owner` moves it under
ontology administration, recording the previous owner and the acting subject.
Four conformance cases on both stores: the first writer claims; a second
producer is refused for an insert *and* for an overwrite; a transfer moves the
right to write while leaving provenance alone; an owned node's `source`-shaped
payload claims nothing.

**Two decisions worth stating.**
- *The registry is the authority, not `node.owner_principal`.* The column is
  immutable provenance, so consulting it would make a transfer unusable — the
  new owner could not touch a row the old one created. DESIGN said as much
  ("the authority the comparison consults") and now says why.
- *The denial is `permission_denied`, not `not_found`.* Everywhere else in this
  gear a denial is indistinguishable from absence (anti-enumeration). Here the
  caller named a namespace whose owner is a fact about the tenant rather than
  about them, and "not found" would send them to create what exists. That is
  DESIGN's own error-table row, and `SOURCE_NAMESPACE_FORBIDDEN` had been
  declared in the reason vocabulary since the first commit with nothing
  producing it.

**Two doc gaps closed with it.** The `source_namespace_owner` table was named
in DESIGN § 3.7 and never given columns — specified now, including why a claim
is a no-op upsert rather than `DO NOTHING` (it takes the row lock, so
simultaneous claims serialize). And the two new operations are in the § 3.3 REST
table.

**What is still thin.** The fake store enforces the boundary and carries the
registry, but does not model the two node columns, because no read surface
exposes them (the same blind spot as D-022). `ingest_audit`, which DESIGN names
as where a transfer is audited, does not exist in this iteration; the transfer
is audited on the registry row instead — previous owner, timestamp, acting
subject — which is enough to answer "who moved this and from whom" and not
enough to answer "how many times".

## D-033 [was deferred as D-109, now built] The readiness surface

`GET /health/ready` answered 404 and DESIGN's 14-row matrix was normative, so
the whole of `fr-readiness` was missing while every *input* it needs was
already being computed at boot. Built: per-component state, each with what it
blocks and the condition being waited on, unauthenticated (the matrix keeps the
health endpoints answering precisely when the authorization resolver is what is
down) and always `200` — readiness is a state to read, not a request that
failed.

**Three decisions the matrix forced.**

- *The aggregate rule and one row contradict each other.* "The aggregate is
  ready only when no component is `Unhealthy`" — and the embedding-space row
  says the identity mismatch is `Unhealthy` while "the gear stays ready". Both
  cannot hold. Resolved in favour of the row, because it is the specific
  statement and the operationally right one: a mismatch blocks the vector arms
  and nothing else, and taking the graph out of service for it would be worse
  than the fault. The aggregate therefore asks the *component* whether its
  failure blocks everything, not the state alone. DESIGN should pick one; this
  entry is the proposal.
- *A fourth state, `not_implemented`.* Five of the matrix's components do not
  exist in this build — the authz probe (the platform PEP publishes no health
  surface), the types registry as a runtime dependency (D-013), dynamic indexes
  (D-104), tenant reconciliation (D-106), metric annotation (D-107). Reporting
  them `Healthy` would be a lie an operator acts on and omitting them would
  hide capabilities they are entitled to ask about, so each is reported absent
  with the deviation that explains why and what would change it. A readiness
  surface that only ever says "fine" is a green light, not a readiness surface.
- *SQL/PGQ is reported `Degraded`, never `Unhealthy`.* The matrix reserves
  unhealthy for a backend an operator *explicitly demanded* and the server
  cannot provide. `traversal_hop` has no value meaning "no preference" — `pgq`
  is both the default and the only way to ask for it — so the gear cannot tell
  a demand from a default and does not pretend to. Either the config grows an
  `auto`, or the matrix's row loses its distinction; until then the fallback
  serves and readiness says so.

**What it does not report.** The server major, for the reason D-004 gives: the
probe is an attempted pattern, which says the pattern did not run and not why.

## D-034 [was a silent cut, now built] Scope replacement removed nothing

*(Written when one function did both halves, under the name
`fence_and_clear_scope`. The halves have been named apart since: fencing is
`fence_scope` (`src/infra/store/ingest.rs`) and the removal is
`src/infra/store/scope.rs::remove_stale`. The old name is kept below where it
describes what the code did at the time, and nowhere else.)*

`fence_and_clear_scope` wrote the fence row, returned `(0, 0)`, and carried a
comment saying full declarative replacement was a scope cut — which no entry
recorded until the acceptance sweep found it (criterion 2, "removes stale
static content and preserves analysis edges"). The fencing half was real; the
replacement was not, so the half of the criterion about *preserving* analysis
edges had nothing to preserve them from.

**Built.** `infra/store/scope.rs` runs after the batch's own writes and inside
the same transaction, under the fence row's lock, because "absent from the
submitted batch" cannot be decided until the batch is in. What it removes is
bounded three ways, and each bound is the feature rather than a detail of it:
scope-managed types only (`scope_managed` is per type and defaults to true);
only what the batch did not re-supply (membership is the payload attribute the
replacement names, so anything re-written is still in scope); and static
content only — static edges first, then only those nodes nothing references any
more, so a node an analysis edge still points at stays. That last predicate is
`principle-provenance-survives-resync` made executable, and it is also what
keeps the `ON DELETE RESTRICT` foreign key from being the thing that decides.

**Removal is a hard delete, not a tombstone.** A tombstoned node key is not
reusable before purge (Soft Delete Contract), so tombstoning here would make
the *next* import of the same object a conflict — the opposite of what a
replacement is for. The conformance case asserts the re-add, not just the
removal.

**The scope attribute is rendered as a checked literal**, the same alphabet a
declared `index` path must use, with the value bound — an attribute outside it
is refused rather than escaped.

**Covered from the same obligation, and covering it found the fence broken.**
Single-writer serialization per scope identity now has its case:
`two_replacements_of_one_scope_serialize` (`tests/conformance/mod.rs`, wired
into both `tests/fake_conformance.rs` and `tests/pg_conformance.rs`) races two
replacements of one scope on a multi-threaded runtime. It is recorded here not
because it passes but because of what writing it found — the fence was
comparing and writing in three steps with no lock between them, so it held
against a sequential retry and not against the case it exists for. That is
**D-035**, which carries the defect, the fix and why the fix is the only lock
a gear can take.

## D-035 [impl-gap] The scope fence compared and wrote in three steps, with no lock between them

Obligation 2 of the store contract — "replacements of one scope serialize on
that identity through a lock held to commit, and the highest accepted
generation is compared and updated atomically under that lock" — was
implemented as read, decide, write. Two concurrent replacements therefore both
read the old generation, both passed the check, and the loser's *lower*
generation overwrote the winner's: the fence was real against a sequential
retry and absent against the case it exists for.

The suite's own header had claimed this obligation since the first commit with
no case behind it, which is how the gap survived: the sequential fencing case
passes either way.

**Fixed** by making the compare *be* the write:
`ON CONFLICT DO UPDATE SET generation = GREATEST(stored, offered)` keeps the
higher generation whoever arrives second and takes the row lock for the rest of
the transaction — so the batch's writes and the stale removal all happen under
it, which is what serializes the two replacements. The row is read back
afterwards and the decision is made on that settled value: higher stored than
offered is stale, equal with another hash is a conflict, equal with ours is
accepted.

**This is not a clever alternative to a lock — it is the only lock available.**
The platform's secure ORM exposes no row-locking surface at all: `SELECT …
FOR UPDATE` cannot be expressed from a gear, and there is no `lock_exclusive`
on the query builders. Worth raising with the platform alongside the other two
asks, because the next gear that needs a fence will write the same three steps
and not notice.

**Covered** by `two_replacements_of_one_scope_serialize`, on both stores and on
a multi-threaded runtime — on the default single-threaded one the two futures
only interleave at await points, which is not the race in question. The
assertion holds whichever reaches the fence first: the higher generation's
content is what remains, the lower one's node is never there beside it, and the
recorded generation is the higher.

## D-036 [impl] The fake numbered nodes per tenant, which made a leak test pass vacuously

`FakeGraphStore` allocated internal ids from a counter held **inside each tenant**, so the first node of tenant A and the first node of tenant B both had id 1. The built-in store's ids come from a PostgreSQL sequence and are unique across the whole database.

It never mattered until the adversarial sweep hydrated by a *foreign* internal id -- the one read surface where a missing tenant predicate does not show up as a key collision, because nothing about the request mentions a key. On the fake the foreign id was our own node's id, so the case got a row back, and the row it got back was the right one: the assertion would have passed whatever `hydrate_nodes` did with the tenant.

**Implementation:** the counter moved to the store as an `AtomicI64`. Nothing else changed; ids are opaque to every caller.

**Worth stating generally:** a fake diverging from the real store is expected and fine where the divergence is declared (snapshots, D-007). This one was undeclared and invisible, and it disarmed a test rather than failing one. The conformance suite is the only thing that can catch that class, and only for behaviour it actually exercises.

## D-037 [impl-gap, fixed] A type whose schema cannot compile registered successfully and failed every write

Registration validated the derivation chain from the **identifier** -- which is where the chain lives -- and never compiled the schema body. A `$ref` to something nobody registered therefore passed registration and failed at the first ingest of that type, with `schema does not compile: unresolved schema reference ...`. The producer got a success for the act that was wrong and a failure for every act that was right.

Found by driving registration through the domain service for the first time (`tests/service.rs`).

**Implementation:** both stores now compile the chain validator at registration, immediately after `analyze`, and report a compile failure as a per-item validation error. It is the same validator ingest compiles, so a type that registers is a type that can admit an instance.

**Two things fell out of it.** The base ontology is now always resolvable inside `ChainValidator::compile`, not only when it happens to be on the chain: an analysis edge references the *provenance attribute*, a sibling family that is never an ancestor, so compiling the analysis-edge family type failed while compiling a leaf derived from it succeeded. And the conformance suite's own phantom case carried a misspelled `$ref` (`gts://gts.cf.core.graph.reference_node.v1~`, missing the `node.v1~` segment) which nothing had ever resolved, because nothing had ever compiled it.

**Proposal:** none for the documents -- `fr-type-registration` already asks for a registration that validates. This is the implementation catching up with it.

## D-038 [impl-gap, fixed] Coverage was never measured, and four surfaces had no test at all

`nfr-code-coverage` sets 85 %. `cargo llvm-cov` had never been run against this gear, so the number was unknown and, more to the point, so was *what* was uncovered. The first run said 73.25 % regions / 71.66 % lines, and the shape of the gap was more interesting than the number:

| surface | lines covered before |
| --- | --- |
| `domain/service.rs` -- the layer every request goes through | 0 % |
| `api/rest/{handlers,dto,error}.rs` | 0 % |
| `domain/local_client.rs` -- the `ClientHub` entrance | 0 % |
| `domain/admission.rs` -- the Capacity and Admission Contract | 0 % |
| `infra/store/evolution.rs`, edge branch | 0 % |
| `infra/store/spaces.rs` -- boot-time embedding-space resolution | 0 % |

The conformance suite calls `GraphStoreV1` directly, which is deliberate and right; the consequence nobody had drawn is that everything *above* the port was unexercised, including the admission bounds that are the gear's half of the capacity contract.

**Implementation:** `tests/service.rs` (the real `GraphServices`, a real `PolicyEnforcer` over a stub PDP, a one-hop engine over the in-memory store), `tests/rest.rs` (real requests through the gear's own axum router), an exhaustive `DomainError` to `CanonicalError` mapping test, a local-client parity case, an edge-type evolution case on both stores, and a boot-resolution case on a real server.

**After: 86.77 % regions, 87.02 % lines, 83.41 % functions.** The remaining uncovered block of any size is `gear.rs` (the composition root, which needs a platform to boot) and the refusal branches of `store/types.rs`.

**What the writing of them found** is recorded separately: [D-037](#d-037-impl-gap-fixed-a-type-whose-schema-cannot-compile-registered-successfully-and-failed-every-write). Worth saying once here: a coverage number is a poor goal and a good instrument. Chasing 85 % is how the fact that no test had ever sent an HTTP request to this gear became visible.

**Found later:** the 85 % this entry chases is not the number CI fails on. `tools/scripts/coverage.py` gates at a project-wide 80 with no override for this gear, so the gear is over both thresholds and the stricter one is currently kept by hand — D-048.

## D-039 [impl-gap] The ingest producer principal never reaches the store, so two rules are narrower than they read

- **Doc:** PRD `fr-bulk-ingest` — "every ingest request carries a **tenant- and producer-scoped** idempotency key"; PRD `fr-scope-replace` — "a scope **MUST** have a canonical identity (tenant, **owning producer**, scope attribute and value)"; DESIGN § Concurrent Ingest Protocol rule 3, which spells the same identity out and hangs the replacement lock on it.
- **Implementation:** `src/infra/store/ingest.rs:284` — `let producer = String::new();`. That empty string is then the value written into `ingest_idempotency.producer` (`ingest.rs:409`), the value filtered on when a receipt is replayed (`ingest.rs:1514`), and the value written into and compared against `scope_registry.owner_producer` (`ingest.rs:463`, `ingest.rs:511`).
- **Why it matters, in two different ways.** The idempotency key becomes **tenant-scoped**: two producers in one tenant that happen to choose the same key collide, and the second one either replays the first's response or is refused as a hash mismatch, for a request it never sent. And the fence's ownership check at `ingest.rs:511` — `if row.owner_producer != producer` — compares `""` with `""` on every path, so it can never fire: **any writer in the tenant can replace any scope**, including one another producer claimed and is re-syncing. The fencing that D-034 and D-035 made real is generation fencing; the *ownership* half of the same identity is vacuous. That is a p1 authorization gap, not a diagnostics one, and it is the same shape as D-032: a column parsed, stored, compared against itself, and read by nothing that could tell.
- **The principal was available the whole time.** `Subject::principal()` is what the same file already calls twelve lines away, at `ingest.rs:814` and `ingest.rs:837`, to write `node.owner_principal` for the source-namespace boundary. Nothing was missing from the port; the value was simply never threaded into the two places that needed it.
- **Proposal:** carry `ctx.subject.principal()` as the producer into the receipt and the fence, and decide explicitly what a namespace transfer means for a scope the previous owner still holds — the source-namespace registry already answers the equivalent question for nodes (D-032), and the two ownership notions should not be allowed to drift apart. **Being fixed separately**; recorded here because the gap predates the fix and because the way it survived — a boundary implemented against a constant — is the part worth remembering.
- **How it was checked:** read at the assignment and followed to all four uses; `grep` for `principal()` shows it reaching the node columns and nothing else. No test could have caught it: every conformance case ingests as one subject, so `"" == ""` and the real comparison are indistinguishable from the outside.

## D-040 [impl-gap] Writes are one statement per row, where the PRD requires batched statements

- **Doc:** PRD `fr-bulk-ingest`: "Writes **MUST** use batched database statements", with the rationale naming the reason — "the prototype's row-at-a-time writes were a measured bottleneck". DESIGN § 2 repeats it in the component table ("writes nodes/edges/chunks with batched statements in one transaction").
- **Implementation:** row at a time. `write_nodes` (`src/infra/store/ingest.rs:1295`) loops the batch and calls `upsert_node` (`ingest.rs:772`) per node; `write_edges` (`ingest.rs:1419`) loops and calls `upsert_edge` (`ingest.rs:1026`) per edge, after resolving each endpoint and reading the endpoint types for the constraint check. Each of those is a `SELECT` followed by an `INSERT` or an `UPDATE`, so a 20 000-edge batch is tens of thousands of round trips inside one transaction. Atomicity — the other half of the requirement — does hold: it is all one transaction.
- **Why it matters:** the perf lane measures it. 10 000 nodes + 20 000 edges lands in 19.7 s against a 60 s budget, so nothing published is at risk today, but seeding the 600 000-row reference graph took 810 s and was markedly non-linear — roughly a thousand edges a second at the start and under a hundred a second around the 450 000 mark (`dev/PERF-6.1.md`). A producer re-syncing a repository into an already-large graph is on the slow part of that curve, and the requirement exists precisely because v1 was there before.
- **Proposal:** batch the reads first — one `SELECT` resolving every key of the batch, which is where most of the round trips are — then multi-row `INSERT … ON CONFLICT DO UPDATE` for the writes. The per-row decisions that stand in the way are the tombstone conflict, the type-immutability check and the endpoint-constraint check; each of them can be decided from the batched read rather than from a read of its own. Worth doing before anyone promises a bulk-import SLA, and worth measuring rather than assuming, since the non-linearity above has not been attributed yet.
- **How it was checked:** read both write loops and the two upserts they call; the shape is confirmed by the seeding curve in the perf note, which is what a per-row statement count against a growing index looks like.

## D-041 [impl-gap] `nfr-response-bound` has no configuration key and nothing truncates before hydration

- **Doc:** PRD `nfr-response-bound` (p1): "Every response **MUST** be bounded in aggregate, not only per item: cumulative hydrated payload bytes, returned edge count, snippet/chunk-provenance/annotation bytes, and total serialized bytes each have a **configured ceiling enforced in the domain layer before hydration**, with deterministic truncation … and explicit truncation metadata in the response. REST and the in-process client are bound by the same values." Its rationale is explicit that per-item ceilings do not compose.
- **Implementation:** the per-*item* and per-*cardinality* bounds exist and are enforced in `src/domain/admission.rs` — `ingest_max_nodes`, `ingest_max_edges`, `payload_max_bytes`, `item_max_bytes`, `node_read_max_adjacency`, the four traversal bounds, `search_max_arm_limit`, `projection_max_page` (`src/config.rs:87-97`). **No aggregate byte ceiling exists at all**: there is no key for cumulative hydrated bytes, no key for total serialized bytes, nothing sums either, and nothing truncates between admission and hydration. What bounds a response in practice is cardinality — page size, node budget, arm limit — multiplied by whatever `payload_max_bytes` allows per row, which is exactly the composition the requirement says is not a bound.
- **Why it matters:** the worst case is a page of admissible rows whose payloads are each under the per-item ceiling. At `projection_max_page` rows of `payload_max_bytes` each the arithmetic is megabytes, discovered while serializing — the failure mode the rationale names — and a traversal hydrating its whole node budget has the same shape. It is a denial-of-service surface reachable with entirely valid requests, and it is also why `TruncationReason` has no aggregate-bytes variant: nothing can report a truncation that never happens.
- **Proposal:** two keys (cumulative hydrated bytes, total serialized bytes) enforced in the domain layer at the point where the candidate set is known and before hydration, truncating at the ordering the query already established and reporting it as a truncation reason — which means a new `TruncationReason` variant and a field on the projection and search responses, since only traversal carries one today. The `p1` marking means this is release-blocking rather than a prototype gap; nothing about it needs platform work.
- **How it was checked:** every key in `src/config.rs` read against the requirement's list; `src/domain/admission.rs` read for a byte accumulator (there is none); the hydration paths in `domain/service.rs` read for a pre-hydration cut (there is none).

## D-042 [impl-gap] The neighborhood projection has no degree ordering

- **Doc:** PRD `fr-neighborhood-projection` (p1): return the connected subgraph within a node budget, "**ordering retained nodes by degree so truncation keeps the structural core**". DESIGN § Authorization Model restates it from the other side — "degree ordering, budgets, and truncation are computed on authorized rows only" — so the ordering is assumed by the security contract as well as by the feature.
- **Implementation:** `GraphServices::neighborhood` (`src/domain/service.rs:837`) admits the request, builds a `WalkPlan`, makes a one-element seed list from the root and hands it to the same `walk_and_hydrate` the traversal uses. `domain::traversal::walk` is breadth-first and its truncation is **arrival order**: when `ordered.len()` reaches `max_nodes` it sets `TruncationReason::NodeBudget` and stops (`src/domain/traversal.rs:91-97`). No degree is computed anywhere in the gear — `grep -rn degree src/` returns nothing.
- **Why it matters:** on a dense graph the difference is the whole feature. A depth-3 neighborhood of a hub node truncated by arrival order keeps whichever leaves the first hop happened to return and drops the structurally central nodes that make the picture readable; the criterion's phrase "keeps the structural core" is the requirement, not a decoration. It is a silently-wrong-answer gap rather than a failure: the response is well-formed, bounded, correctly scoped, and reports that it truncated — it just keeps the wrong nodes, and no caller can tell.
- **Proposal:** compute degree over authorized rows (the engine already reads the incident edges it would need) and order the retained set by it before the budget cuts, with the root exempt as seeds are. Cheap while the frontier is bounded by `traversal_max_frontier`; the ordering has to happen after scoping, per the Authorization Model, so it belongs in the domain walk rather than in the engine.
- **How it was checked:** the walk read end to end for an ordering step; `grep` for `degree` across the gear finds no producer and no consumer.

## D-043 [impl-gap] Traversal takes seed keys only, reports no seeds, and bounds them before it authorizes them

- **Doc:** PRD `fr-graph-traversal` (p1) is unusually specific: seeds are "given as explicit node keys, **as hybrid-search hits for a query, or both**"; responses "**MUST** include the traversed nodes, edges, **seeds**, and truncation status"; and, because seeds survive truncation, "**after authorization and deduplication**, a request whose distinct authorized seeds exceed the effective node budget **MUST** be rejected … and seed ordering and **admitted-seed metadata MUST** be deterministic."
- **Implementation:** three divergences, in one path.
  - *Seeds are keys only.* `TraverseRequest.seeds: Vec<NodeKey>` (`graph-storage-sdk/src/models.rs`), and there is no field for a query. A caller wanting "search then expand" — the scenario the PRD's rationale says motivated the gear — makes two calls and joins them, which is exactly the flow the criterion's own § 6.1 scenario 1 describes as one.
  - *The response carries no seed list and no admitted-seed count.* `TraversalResponse` is `nodes`, `edges`, `truncated`, `revision`. The seeds are in `nodes` (they are walked first, and the walk sorts and dedups them, so the order is deterministic) but they are not distinguishable from anything else the walk reached, and nothing reports how many were admitted.
  - *The bound is applied to the wrong set.* `admission::admit_traverse` refuses when `request.seeds.len() > max_nodes` (`src/domain/admission.rs:107-111`) — the **raw** submitted count, before the dedup the walk performs and before authorization drops the seeds this caller cannot see. So a request with 600 seeds against a 500-node budget is refused even when it names 200 distinct authorized nodes, and the requirement's wording ("distinct authorized seeds") is not what is measured.
- **Why it matters:** the third one refuses requests the specification admits, which a producer experiences as an arbitrary limit; the second leaves a caller unable to tell a seed from a reached node, which matters precisely because seeds are budget-exempt; the first is a missing feature with a documented use case. None of them returns a wrong answer, which is why they survived — the paths that *are* built are correct.
- **Proposal:** resolve and authorize the seed set first, dedup it, bound the result, and report `seeds` and an admitted-seed count on the response. Search-derived seeds are a larger change (a search request nested in a traversal request, and one snapshot across both) and are the natural home for the search-then-expand scenario; they should be scoped deliberately rather than added to the DTO.
- **How it was checked:** the DTOs read in the SDK, the admission check read at its line, and the walk read for the ordering guarantee (which does hold).

## D-044 [impl-gap] The edge-scan budget is per hop, and a hop that reaches it truncates silently

- **Doc:** DESIGN's traversal section makes the scan budget part of a *walk's* bound, alongside depth and the node budget, and `TruncationReason` is documented in the SDK as "why an expansion or traversal stopped early. **Never silent.**"
- **Implementation:** `domain::traversal::walk` passes the same `plan.max_edges_scanned` into every hop's `HopBudget` (`src/domain/traversal.rs:73`), so a depth-3 walk may scan three times the configured budget; nothing accumulates across hops. And in the engine the budget is applied as `.limit(req.budget.max_edges_scanned)` on the incidence read (`src/infra/engine.rs:439`), after which the only truncation the hop can report is the frontier cap (`engine.rs:370`, `engine.rs:524`, both computed from the reached-node count). A hop that reads exactly `max_edges_scanned` rows is indistinguishable from one that read every edge there was: `TruncationReason::EdgeScanCap` exists in the enum and has exactly one occurrence in the gear — the REST serializer's match arm (`src/api/rest/dto.rs:894`) — and no producer anywhere.
- **Why it matters:** this is the silent-wrong-answer case in the traversal path. A dense hub whose incident edges exceed the budget returns a subgraph that is arbitrarily cut at whatever order the index happened to return, reported as complete, and a caller counting or drawing it is wrong in the way D-014 describes for double-counted edges — only without the double-count to notice. The cumulative question is smaller but real: `traversal_max_edges_scanned` is documented as the walk's protection against a runaway traversal, and at depth 3 it protects three times as much work as its value says.
- **Proposal:** carry the remaining budget through the walk and decrement it per hop, and have the engine report `EdgeScanCap` when the incidence read returns its limit — `limit + 1` and a comparison, the same device the frontier cap already uses in `engine.rs:336`. Both halves are local to the two files.
- **How it was checked:** the budget followed from `WalkPlan` through every `ExpandRequest`; `grep` for `EdgeScanCap` across `src/` and `tests/` finds only the serializer arm, so nothing can ever emit it.

## D-045 [impl-gap] A search hit carries no element envelope

- **Doc:** PRD `fr-audit-envelope` (p1): the gear-assigned envelope is on "**every element read**", and DESIGN § API element envelope defines it as a property of any node or edge a read surface returns. D-022 closed the edge half of exactly this requirement on exactly this reasoning.
- **Implementation:** `SearchHit` is `node_key`, `type_id`, `name`, `score`, `arms` and `snippet` (`graph-storage-sdk/src/models.rs:906`). No envelope, no revision on the element — the revision is on `SearchResponse` instead, which is the one place search does report it. So a caller who searches and wants to know when a hit was last written, or by whom, has to read each hit back individually.
- **Why it matters:** whether it is a divergence depends on whether a hit is "an element", and the honest answer is that it is a ranked *reference* to one — which is the same argument that was made for `EdgeRef` and `AdjacencyEntry` in D-022, and it was the right call there. The reason to record it anyway is D-022's own lesson: the requirement says every node any read surface returns, a hit carries a payload-free identity today, and the moment a hit grows a payload or a hydrated field the distinction stops being defensible and nothing tests the boundary. Recording it also makes the decision visible rather than incidental.
- **Proposal:** decide it explicitly in DESIGN — either a hit is a reference and the envelope belongs on the hydration that follows (in which case say so beside the envelope contract, as D-024's repetition needs saying), or hits carry the envelope like projection rows do (D-021), which costs one join and makes "every read surface" true without an asterisk.
- **How it was checked:** the DTO read in the SDK and the REST serializer read beside it; no envelope reaches a hit on either path.

## D-046 [impl-gap] Type-family filtering has no test, and the authorizing permission's pattern is not intersected with the caller's

- **Doc:** DESIGN § Authorization Model, "GTS pattern resolution is shared, and never text matching": *"Two pattern filters meet over the same type column on one request: the caller's type-family filter and the `resource_type` of the permission that authorized them, which may itself be a GTS wildcard pattern. Both resolve through `GtsIdPattern` … and the request's effective type set is **the intersection of the two sets**."*
- **Implementation:** half of it. The caller's half works and is real — `search` resolves `request.type_patterns` through the shared resolver (`src/infra/store/search.rs:39-42`), and traversal resolves `edge_type_patterns` and `node_type_patterns` the same way (`src/domain/service.rs:818-822`). The permission's half does not exist: `domain::authz::scope_for` asks the PEP for an `AccessScope` and returns it (`src/domain/authz.rs`), and an `AccessScope` is a row predicate — it carries no type pattern the gear could resolve and intersect. So a permission granted over a GTS wildcard authorizes the caller for the resource and then constrains nothing about *which types* they may read, and the effective type set is the caller's filter alone.
- **And it is untested on both halves.** Every call site in every suite passes an empty pattern list: `type_patterns: Vec::new()` and `edge_type_patterns`/`node_type_patterns: Vec::new()` at all fourteen occurrences across `tests/service.rs`, `tests/conformance/mod.rs`, `tests/rest.rs` and `tests/perf.rs`. Nothing has ever asked this gear to filter by type family — not the pattern resolution, not the implicit derived-type coverage a bare base identifier carries, not the refusal of an unresolvable pattern. That is the one entirely untested feature surface left in the read paths.
- **Why it matters:** the untested half is a correctness risk on a feature the § 6.1 scenarios all use (each names a type filter). The unimplemented half is an authorization risk of the D-039 shape: the document describes a narrowing that does not happen, so a reader — or an administrator granting a deliberately narrow pattern permission — believes the type set is constrained when only the row set is.
- **Proposal:** conformance cases on both stores for a pattern that selects a family, a pattern that selects a leaf, a bare base identifier (which must cover its descendants), and an unresolvable pattern; and then either implement the intersection — which needs the PEP to hand back the authorizing permission's `resource_type`, a platform ask — or amend DESIGN to say the type set is the caller's alone and that a narrow pattern permission does not restrict types. The first is the better shape; the second must happen if the first cannot, because the paragraph as written is not true of the code.
- **How it was checked:** `grep -rn type_patterns src/ tests/` — every test occurrence is an empty vector; `authz.rs` read end to end for a pattern the enforcer returns (it returns only the scope).

## D-047 [impl-gap] `idempotency_retention_days` is a key nothing reads

- **Doc:** DESIGN § Concurrent Ingest Protocol rule 2: "Idempotency records are retained for a configurable window (`limits.idempotency_retention`, default 7 days)", and DESIGN's configuration table gives the key, the default, the range, and its enforcement point — "Background cleanup".
- **Implementation:** `idempotency_retention_days` is declared (`src/config.rs:101`), defaulted to 7 (`config.rs:167`) and range-checked 1..=365 (`config.rs:239`). Those are its only three occurrences in the gear: nothing reads the value, and there is no cleanup of any kind — the gear has no background task at all (D-031). What actually expires a receipt is the source epoch: `replay_receipt` treats a receipt whose `source_epoch` is not the current one exactly as expired (`src/infra/store/ingest.rs:1528`), and the epoch changes only by operator action, which is itself unimplemented (D-050).
- **Why it matters:** `ingest_idempotency` grows without bound — one row per ingest request per producer per tenant, each holding a canonical request hash and a serialized response. The correctness of a replay is unaffected, and the configured range says 1 to 365 days to an operator who can set it to any value with no effect whatsoever, which is worse than the key not existing. It also means a key is reusable *never* rather than after a week, so a producer that recycles keys across long-running pipelines gets a hash-mismatch conflict instead of a fresh request.
- **Proposal:** the cleanup is a background task, so it waits on the same absent mechanism as the re-embedding backfill and the asynchronous migration (D-031) — which is the argument for building one rather than three. Until then the key should be documented as unenforced rather than offered with a range, and the retention story should say plainly that receipts live until the epoch rotates.
- **How it was checked:** `grep -rn idempotency_retention src/` returns the declaration, the default and the range check, and nothing else; the epoch comparison read at its line.

## D-048 [doc-gap] `nfr-code-coverage` says 85 % enforced in CI, and CI enforces 80 %

- **Doc:** PRD `nfr-code-coverage` (p1): "The gear **MUST** maintain at least 85% line coverage across its library crates", and PRD § 5's testing-strategy note calls it "the enforced floor … gated in CI".
- **Implementation:** `tools/scripts/coverage.py` carries `COVERAGE_THRESHOLD = 80` as a module constant (line 35) and threads it through every report and gate; it takes a `--threshold` argument but has no per-gear configuration and no override for this gear. So the number CI fails on here is 80, and the gear's own 85 is a figure in a document. The gear is comfortably over both — 87.02 % lines as of D-038 — which is exactly why nobody noticed.
- **Why it matters:** a coverage number that is *stated* as gated and is not is worse than one stated as a goal: a regression from 87 % to 82 % would pass CI green while breaching a p1 requirement, and the layers that fell to 0 % before D-038 (the whole REST surface, the admission bounds) are the ones a refactor would drop again first. It is also the second instance of this register's recurring pattern — a rule the documents state and nothing enforces — and the cheapest one to close.
- **Proposal:** either give the coverage tool a per-gear floor and set this gear's to 85, or amend `nfr-code-coverage` to the threshold the platform actually gates (80) and record 85 as this gear's own bar with the entry that measures it. The first is right if the requirement is meant; the second is honest if it is not. What must not stand is the current pair, because it reads as enforced.
- **How it was checked:** `tools/scripts/coverage.py` read at the constant and at its call sites; no gear-specific threshold appears anywhere in the repository's coverage configuration.

## D-049 [impl-gap] Deleting a tombstoned row is a 404, and re-ingesting a tombstoned *edge* revives it

- **Doc:** DESIGN § Soft Delete Contract, rules 3 and 4. Rule 3: "The revision moves if and only if state changed. A delete increments the tenant's graph revision; **deleting an already-deleted row is a no-op that leaves it untouched**, exactly as a converging ingest replay does." Rule 4: "A tombstoned `node_key` is not reusable before purge. **Re-ingesting it is a conflict, not a resurrection**" — stated of `node_key`, and the reasoning given ("consumers still hold that key") is not node-specific.
- **Implementation:** two divergences, in opposite directions.
  - *A second delete is `NotFound`.* `soft_delete` looks the row up with `DeletedAt.is_null()` in the filter and maps the miss to `GraphStoreError::NotFound` (`src/infra/store/ingest.rs:1141` and `:1145` for a node, `:1207` and `:1211` for an edge), which surfaces as `404`. The fake does the same (`src/infra/fake_store.rs:776`, `fake_store.rs:794`). The contract describes a no-op — the revision correctly does not move, but the caller is told the row does not exist, which is also what they would be told if it never had.
  - *A tombstoned edge comes back.* `upsert_node` refuses a tombstoned key explicitly and by name (`ingest.rs:860-866`: "node key … is tombstoned and cannot be re-ingested before purge"). `upsert_edge` has no such branch: it treats a tombstoned row as merely changed — `if current.payload == payload && current.deleted_at.is_none()` is the unchanged test (`ingest.rs:1080`) — and the update it then runs clears `deleted_at`, `deleted_by_subject_id` and `deleted_by_subject_type` (`ingest.rs:1097-1108`). So re-asserting a deleted relationship silently resurrects it, with the tombstone's audit trail erased rather than superseded.
- **Why it matters:** the revive is the serious half. An edge is derived — its key is a hash of type, endpoints and discriminator — so a producer re-syncing a source it has not changed re-asserts every edge it ever asserted, and an edge an operator or an analysis deliberately deleted comes back on the next sync with nothing recording that it did. Deletion of an edge is therefore not durable against the ordinary write path, which is not what "tombstone, not a row removal" promises. The 404 is milder but breaks an idempotency property the contract states deliberately: a delete retried after an unknown outcome should converge, and instead it reports a failure.
- **Proposal:** make a second delete a no-op that reports the row's existing tombstone (the revision already stays put), and decide the edge revive explicitly rather than by omission — either refuse it as `upsert_node` does, or define a re-assertion as a deliberate undelete that records the reviving subject and says so in the contract. Silence is the one option the Soft Delete Contract does not leave open, since it is the half of the contract consumers rely on.
- **How it was checked:** all four paths read in both implementations; the asymmetry between `upsert_node`'s explicit refusal and `upsert_edge`'s missing branch is visible in the same file, twenty lines apart. Not covered by any test: `tombstoned_rows_are_absent_from_every_read_path` asserts the read side, and no case deletes twice or re-ingests a deleted edge.

## D-050 [impl-gap] There is no epoch-rotation mechanism, so half of `fr-snapshot-identity` has no trigger

- **Doc:** PRD `fr-snapshot-identity` (p1): `(source_epoch, graph_revision)` is the snapshot identity in continuation tokens, cache keys, job identity and plugin cursors, and the epoch is "**rotated by operator action before ready after restore**". `graph_meta`'s own docstring restates it: "rotated by operator action after a restore or store replacement" (`src/infra/storage/entity/graph_meta.rs:6`).
- **Implementation:** the epoch is read everywhere and written once. `graph_meta` is seeded with `source_epoch = 1` at first boot (`src/infra/store/ingest.rs:1256`); the only other occurrences of the key are reads — the revision read (`src/infra/store/reads.rs:101`) and the ingest path's own lookup (`ingest.rs:231`). There is no REST route, no administrative operation, no configuration key and no boot path that changes it. `grep -rni rotat src/` finds two comments describing the rotation and no code performing it.
- **Why it matters:** an epoch that never moves makes the *whole* of the snapshot identity a revision counter, and the thing the epoch exists to invalidate is a restore. Restore a database from a backup and the revision counter goes backwards while continuation tokens, cache keys and plugin cursors minted against the old timeline still parse and still look current — which is precisely the confusion the pair was designed to prevent. What the epoch does work for is the one consumer that reads it as a condition rather than as an identity: an idempotency receipt from another epoch is treated as expired (D-047), so *if* an operator could rotate it, receipts would correctly stop replaying. They cannot, so that safety is latent.
- **Proposal:** one administrative operation under ontology administration that increments the epoch and refuses while requests are in flight, plus the readiness interlock the requirement names — readiness withheld until the operator has rotated after a restore, which needs a durable marker that a restore happened. The operation is small; the interlock is the part that needs designing, and it belongs with the tenant-reconciliation row of the readiness matrix that is already reported `not_implemented` (D-106).
- **How it was checked:** every occurrence of `KEY_SOURCE_EPOCH` read (one write at first boot, two reads); no route, config key or service method touches it.

## D-051 [doc-gap] The base ontology is published to one registry, not two — and D-013 over-claimed it

- **Doc:** DESIGN § Base Ontology Publication, first paragraph: "**At startup** the gear registers the base ontology defined in § 3.1 … and its permission instances **with the platform types-registry through the standard inventory mechanism**, so producers can derive types and administrators can grant permissions before any runtime registration happens." Its "Found while building the prototype" note then says publication "happens **twice, in two registries**, at two moments" — the platform registry at startup, the gear's own per-tenant projection at a tenant's first registration.
- **Implementation:** the second moment exists and the first does not. `with_base_ontology` prepends whichever base schemas a tenant is missing to that tenant's first registration batch, exactly as described. There is no types-registry client in the gear: `grep -rni 'inventory|types_registry|register_gts' src/` finds no client, no inventory declaration and no startup publication, and `GraphStorageGear::init` (`src/gear.rs:40`) resolves configuration, the embedding provider and the embedding space, and registers migrations and REST routes — nothing else. `gts_type`'s docstring already calls the table "a per-tenant projection of the platform types-registry", and the projection has no source.
- **Why it matters:** the paragraph's purpose is the sentence at the end of it — "so producers can derive types and administrators can grant permissions **before** any runtime registration happens". Neither is possible. A producer cannot browse the base ontology to see what to derive from, and an administrator cannot grant a permission over a type the registry has never heard of; the schemas exist only inside the gear's crate (D-026) and inside each tenant's own projection after that tenant has already registered something. It is also the reason the readiness matrix reports the types registry `not_implemented` rather than healthy (D-033), and the reason the evolution verdict cannot yet be delegated to it (D-031).
- **And it corrects D-013.** That entry's "folded into the docs" line reads: "DESIGN § Base Ontology Publication now says publication happens twice, in two registries, and the gear's own copy is per tenant on first registration." The second half is true and the first half is a description of an intended design, not of this build — publication happens **once**, in the gear's own per-tenant table. D-013's larger finding underneath it (that publication is also the only moment, because a base schema edit can never reach a database that has already published it) is unaffected and stands.
- **Proposal:** either build the startup publication — which needs the platform inventory mechanism and is the thing the readiness row is waiting for — or mark the paragraph as describing the target state and say which half this iteration ships, in the amendment style the rest of the document uses. The second is honest now; the first has to happen before the permission story or the delegated verdict can work.
- **How it was checked:** the gear's `init` read end to end; `grep` for an inventory or registry client across `src/` and `Cargo.toml` finds nothing; the nine schemas traced from `graph-storage/schemas/` to the per-tenant prepend and no further.

## D-052 [impl-gap] The deadline is minted per store call, and no store read consults it

- **Doc:** DESIGN § Deadlines and Cancellation makes the budget a property of the *request*: a deadline is taken once, carried through every stage, and consulted by work that can outlive it. `deadline_interactive_secs` is the interactive ceiling (`src/config.rs:99`).
- **Implementation:** `GraphServices::store_ctx` constructs `budget: RemainingBudget::starting_now(self.config.deadline_interactive())` and a fresh `CancellationToken::new()` on every call (`src/domain/service.rs:94`), and it is called sixteen times across the service. A request that makes several store calls — a compound read that opens a snapshot, reads, hydrates and closes; an ingest that reads embedding state and then writes — therefore gets a **new full budget per call** and an unrelated cancellation token each time, so the clock restarts rather than running down and the token cancels nothing that is actually in flight.
  It matters less than it would, because almost nothing consults either. The only store path that reads `budget` is the type-evolution re-validation scan (`src/infra/store/evolution.rs:48`, checked between batches, which D-031 records as "the first store path in this gear to consult `StoreCtx::budget`"), and the only other consumers are the embedding providers, which pass it into their own calls (`src/domain/embedding.rs:111-220`). **No store read path consults `budget` or `cancel` at all** — not search, not the projection, not a hop, not hydration.
- **Why it matters:** the bounds that do exist are cardinality bounds, and they bound *work*, not *waiting*. A slow database under load makes every read outlive its deadline with nothing noticing, and the request is killed from outside instead: by the gear's own deadline never, and by `api-gateway`'s 30 s `SYNC_TIMEOUT` eventually, which is the constant D-031 already records as the real ceiling. The per-call minting is the more insidious half, because it makes the budget look carried when it is not — a reader of `store_ctx` would reasonably believe a request has one clock.
- **Proposal:** mint the budget and the cancellation token once per request, at the point the security context is resolved, and thread them into every `StoreCtx` the request builds; then have the read paths consult them where they can act — before hydration, between search arms, and between hops, which is where a walk can decide to answer `Deadline` rather than start another round trip. The evolution scan already shows the shape.
- **How it was checked:** `store_ctx` read at its definition and counted at its call sites; `grep` for `budget` and `cancel` across `src/infra/store*` and `src/infra/engine.rs` finds the evolution scan and the traversal *cardinality* budget (`max_frontier`, `max_edges_scanned`), which is a different thing wearing the same word.

---

# Acceptance criteria: what the prototype actually establishes

PRD § 9 is the checklist this gear will be judged against, and nothing here
recorded where the prototype stands against it. Checked one by one, on the
live stand and in the suites, and **re-checked 2026-09-11** after the
landings that followed: every criterion below now names the case or the entry
that settles it, and says where it is met only at the store port. Nothing
below is a divergence from the specification — it is either evidence that a
criterion holds or the distance still to cover, written down so neither is
rediscovered.

**1. Register an ontology, ingest owned nodes, reference nodes and both edge
families, re-run identically for byte-identical state — *met at the store
port*.** The registration and convergence halves held from the start: an
identical batch re-run reports every row unchanged and leaves the revision
where it was. The half this entry recorded as missing — reference nodes with
their `(system, kind, native_id)` key derivation, and analysis edges with
their required `provenance`, validated by code nothing exercised — is now
exercised: `both_node_families_and_both_edge_families_round_trip`
(`tests/conformance/mod.rs`, run against both stores) ingests one batch
carrying every family, reads each node back, and reads every edge of the batch
back through `GET /edges/{edge_key}`'s store operation, which is also what
pins the two surfaces' agreement on how an edge is named (D-022). *At the
store port*: the case drives `GraphStoreV1`, not REST, so what is established
is that the four families round-trip through the store and not that a producer
posting them over HTTP sees the same thing.

**2. Scope replacement removes stale static content and preserves analysis
edges — *met at the store port*, and building it found a second
defect.** When this section was first written the replacement did nothing:
the one function then called `fence_and_clear_scope` wrote the fence row and
returned `(0, 0)`, so the half of the criterion about preserving analysis
edges had nothing to preserve them from — a deliberate cut that no entry had
recorded. Both halves are built now. Fencing is `fence_scope`
(`src/infra/store/ingest.rs`) and removal is
`src/infra/store/scope.rs::remove_stale`, which runs after the batch's own
writes and inside the same transaction, under the fence row's lock, removing
only scope-managed static content the batch did not re-supply and only nodes
nothing still references — so a node an analysis edge points at stays
(**D-034**). Writing the race the criterion's serialization clause implies
then found the fence itself broken, which is **D-035**:
`two_replacements_of_one_scope_serialize` now holds it on both stores. What is
still not covered is the *bounds* — neither row ceiling exists in the fake
store, so the conformance suite covers the grounds on both implementations and
the bounds on neither.

**3. Four retrieval scenarios within the § 6.1 latency thresholds — *met on
developer hardware, not on the reference profile*.** All four answer, and all
four are now timed. Hybrid narrowing answers with *meaning* only since the
gear started computing its own vectors (D-103): while they arrived from the
producer, the vector arm ranked whatever it was handed against whatever it was
handed, and nothing tied the two to one model. The criteria table's main flow
— filtering and ordering by payload attributes — is built too (D-030), so it
no longer answers only its alternative flow.

The timing is an opt-in lane, `tests/perf.rs` behind `GEARS_GRAPH_PERF`
(`GEARS_GRAPH_PERF_SCALE` runs a smaller graph and then asserts no threshold,
because a latency measured on a tenth of the graph is not evidence about the
graph). Run 2026-09-11, release build, PostgreSQL 19 + pgvector, on the graph
the criteria name — 100 000 nodes, 500 000 edges, every node embedded:

| scenario | measured p95 | budget |
| --- | --- | --- |
| hybrid narrowing, arm limit 50 (query embedding excluded, per `nfr-search-latency`) | 41.2 ms | 500 ms |
| criteria table, filter on a declared payload path, page 50 | 4.2 ms | — |
| bounded traversal, depth 3, edge-type filtered, 8 seeds | 511 ms | 1 s |
| depth-3 UI neighborhood, 1 000-node budget, hydrated | 396 ms | 1 s |
| ingest 10 000 nodes + 20 000 edges (embedding excluded) | 19.7 s total | 60 s |

Quoted from [`dev/PERF-6.1.md`](./PERF-6.1.md) rather than re-measured; that
note carries the full run and its caveats. Two of them bear on whether this
criterion is *met* or merely *not failed*: the numbers come from a developer
machine under WSL2 with the database in a container beside the test process,
not the reference deployment configuration the criterion names, so they are a
floor on headroom rather than a certified result; and the stored vectors are
the deterministic fake's, which is what HNSW actually traverses. The lane also
measures the store and engine ports — not the gateway, the PDP round trip or
JSON serialization.

**4. Chain violations, endpoint-constraint violations and wrong-width vectors
rejected with structured per-item errors — *met*, after a fix.** Chain
violations come back as per-item field violations addressed by JSON pointer,
every violation in one response. The wrong-width vector this criterion names
is no longer a producer's to send: vectors are computed by the gear, and the
width check moved to what the provider returns, per batch, where the FR puts
it. A provider whose declared width is not the migrated one fails the boot
instead, which is the earlier and better place for a configuration error. Endpoint constraints were **not enforced at all** when this record was
first written: `src_types` and `dst_types` were resolved into the type's
effective traits and then never read, in either the validation or the write
path, so an edge between endpoints its type forbids committed. Fixed in
`101ffaa79`, which runs the check where DESIGN puts it — inside the ingest
transaction, endpoint rows locked — and defers it for a phantom endpoint to
materialization, per rule 3 of the Phantom Materialization Contract. Two
conformance cases, one per half, pass against both implementations, and both
halves were then exercised on the live stand: the forbidden edge comes back
`400` with `edges[0]/dst_node_key` and `SCHEMA_VIOLATION`, naming the
endpoint's actual type and the patterns the edge type accepts; materializing a
phantom under a type its accumulated edge forbids comes back naming that edge.

The stand also corrected the test. The phantom case first materialized under a
key the reference-node identity rule already refuses, so the store answered
before the revalidation ran — and both refusals are node-family item errors,
so the assertion accepted the wrong one. Caught only because the live run
printed the message. The case now uses a key the identity rule accepts and
asserts on the edge named in the message; removing the revalidation call makes
it fail, which is the check that the first version would have survived.

**5. Adversarial multi-tenant tests, zero cross-tenant data in every endpoint
— *met at the store port; absent at REST*.** The sweep is one fixture and one
case, `no_read_surface_answers_with_another_tenants_rows`
(`tests/conformance/mod.rs:3627`), run against **both** stores. It seeds two
tenants — one node key deliberately colliding — asserts the other tenant's
fixture exists before trusting any absence, and then walks the read surfaces
one by one: the node read and its adjacency, key resolution, hydration by a
*foreign internal id* (the one surface where a missing tenant predicate does
not show up as a key collision, and the one that D-036 found had been passing
vacuously), the tabular projection, both search arms under text the two
tenants share, the analytics topology load, and `embedding_state` — where a
foreign key reading as *known* rather than `None` would make the coordinator
skip work it owes. The edge read has its own case,
`an_edge_whose_endpoint_is_hidden_is_not_readable`, on both stores; the hop has
`a_hop_never_leaves_its_tenant`, on PostgreSQL only, because that is where the
pattern backend exists.

**What is plainly still absent: no adversarial case goes through REST.**
`tests/rest.rs` drives the gear's own axum router and has a denial case
(`a_denied_caller_cannot_tell_denial_from_absence`) but no two-tenant case at
all, so what is established is that the *store port* does not answer with
another tenant's rows — not that the HTTP surface above it cannot be made to.
Every one of these surfaces is scoped by the compiled `AccessScope` the port
receives, which is the reason to expect the REST cases to pass and not a
reason to have skipped them: the criterion says *every endpoint*.

**6. `cfs validate` passes and CI meets the coverage threshold — *met, and
the threshold it is measured against is not the one the PRD states*.**
`cfs validate` passes: 238 artifacts, 0 errors. Coverage has since been
measured, which is **D-038**: the first run said 73.25 % regions / 71.66 %
lines, and the shape of the gap mattered more than the number — every layer
*above* the store port, the REST handlers and the admission bounds included,
was at 0 %. After the tests written to close that gap — one of which found a
defect of its own on the way (D-037) — it stands at **86.77 % regions, 87.02 % lines, 83.41 % functions**,
over the 85 % `nfr-code-coverage` sets. The caveat is a gate, not a number:
`tools/scripts/coverage.py` carries a project-wide `COVERAGE_THRESHOLD = 80`
and no per-gear override, so CI enforces 80 here and the gear's own 85 is
currently kept by hand — see D-048.

## Gaps this sweep found that no entry recorded

- **Scope replacement removes nothing** (criterion 2 above). The fencing is
  real; the replacement is not. *Fixed 2026-09-11 — see D-034.*
- **Endpoint constraints were never enforced** (criterion 4 above). Parsed,
  stored, unread. *Fixed in `101ffaa79`.*
- **The `vector_search` trait was published and unread.** Resolved across the
  derivation chain, stored with the type's effective traits and served through
  `GET /types` with the description "Paths composed into the embedding
  input" — while nothing composed anything from it. A producer reading the
  catalogue was told the paths it declared did something. They now do; found
  while answering a question about how vectorization worked, which is to say
  by looking rather than by any test. *Fixed in `2b74dfef5`.*
- **`embedding_epoch` and `embedding_input_hash` were written `NULL` on every
  path.** Both columns existed from the initial schema and neither was ever
  populated, so the states `fr-embedding-pipeline` requires had nowhere to
  live. *Fixed in `2b74dfef5`.*
- **The conformance suite's vector width was its own invention.** It picked 8,
  the fake accepted it, and every case failed on the first real server with
  `expected 384 dimensions, not 8`. The suite now takes the width the schema
  was migrated with. Two implementations caught this; inspection had not.
  *Fixed in `c1da5fde3`.*
- **Two ONNX provider tests measured nothing.** `MiniLM`'s tokenizer pads to a
  fixed 128, so a same-batch comparison is padded identically either way and
  held with mean pooling ignoring the attention mask entirely — and so did
  `related > unrelated`, by 0.85 to 0.52, because 122 padding vectors make
  everything resemble everything. Masked, the same pair scores 0.72 to 0.02.
  Found by deliberately breaking the mask and watching the tests pass.
  *Fixed in `5c809f83b`.*
- **The domain service's own wiring was covered by nothing.** The service and
  the conformance suite each resolved a type's declared vector paths, and
  while they resolved it separately a service that passed no paths at all
  would have left every test green. *Fixed in `f3c781270`.*
- **Phantom creation in the PostgreSQL store worked only by accident.** It
  resolved the phantom node type from the types the batch itself named — and a
  producer never names it: the type is `x-gts-final` and authored only by the
  gear. Every phantom that ever appeared did so because some node in the same
  batch happened to carry that type. Found because the suite had no phantom
  case at all, which is itself the point: the fake and the store had diverged
  on a documented contract and nothing was watching. *Fixed in `101ffaa79`.*

## And one claim the suite makes about itself

The conformance module's own header lists five obligations "asserted against
both the built-in store and the fake". Four are: batch atomicity, generation
fencing, no-orphan-edges, and the snapshot obligation (asserted on the fake,
and asserted as *declined* on the PostgreSQL store). **Obligation 2 —
single-writer serialization per scope identity, held until durable — had no
case at all.** Two concurrent replacements of one scope were never made to
race. *Fixed 2026-09-11: the case exists, and writing it found that the fence
did not hold — see D-035.*

---

# Deferred scope (agreed before implementation started)

Each of these is a `[deferred]` entry: the docs require it, this iteration
does not ship it, and the API/schema leave room for it.

## D-100 [deferred] Content chunking and heavy-content offload
`fr-content-chunking`, `fr-heavy-content-offload` (PRD §5.3); tables `chunk` + file-storage adapter.

**Corrected after checking.** The first draft claimed the search path "treats node hits only as the degenerate case of chunk folding so the seam exists". It does not: there is no mention of chunks or folding anywhere in the search implementation. The arms rank nodes and RRF fuses them; adding chunks means adding a folding step, not filling in a prepared one.

## D-101 [deferred] Labels
`fr-labels` (PRD §5.2); tables `label`, `label_assignment`; label routes; per-hop label filters in traversal (`ExpandRequest.labels` stays in the plugin API, built-in engine returns `CAPABILITY_UNSUPPORTED`).

**The deferral is uniform across the port, which is more than `ExpandRequest.labels`.** This entry used to name the traversal filter alone, and `GraphStoreV1` carries four label operations of its own: `upsert_label`, `delete_label`, `list_labels` and `assign_labels` (`graph-storage-sdk/src/plugin_api.rs:243-258`). **Both** implementations answer all four the same way — `Err(GraphStoreError::Unsupported { what: "labels" })` in `src/infra/store.rs:307-333` and in `src/infra/fake_store.rs:810-836` — which the domain maps to `CAPABILITY_UNSUPPORTED`, and no REST route reaches any of them. So a plugin author reading the trait sees the whole label contract present and refusing in one voice, rather than three arms refusing and a fourth quietly returning an empty list, which is the shape that would make "labels are deferred" look like "this tenant has no labels". **How it was checked:** every arm of both implementations read; there is no branch in either that writes a label row.

## D-102 [deferred] Change events via transactional outbox
`fr-change-events` (PRD §5.2). The `emit_events` trait is stored with effective traits; nothing is published.

## D-103 [was deferred, now built] Embedding pipeline and the in-process ONNX provider
`fr-embedding-pipeline` (PRD §5.4), `fr-embedding-dim-guard`, `fr-vector-search`, ADR-0004.

*(Renumbered in place: this entry cited ADR-0005 throughout until a verification pass caught it. The embedding-provider decision is ADR-0004; ADR-0005 is SQL/PGQ access. Same correction as D-019.)*

**What this entry used to record.** Embeddings were producer-supplied and dimension-guarded on ingest; `EmbeddingProviderV1` shipped as a trait with no implementation anywhere in the repository; the `embedding_space` table was deferred, so the identity half of `fr-embedding-dim-guard` was absent. That is no longer the state, and the entry is kept rather than deleted because what it was hiding is worth reading.

**Producer-supplied vectors were not a small deferral.** They are option **C** of ADR-0004 — the option it rejects, in the words *"nothing enforces that all producers and the query side use the same model — mixed-model vector spaces silently break similarity ranking, and the gear cannot embed query text at all without a model"*. The prototype had exactly that shape: `NodeSpec.embedding` and `SearchRequest.query_vector`, width-checked and otherwise unexamined. Two producers with different models would have ranked against each other with nothing to detect it.

**Built (`99e3c87bf`, `2b74dfef5`, `5c809f83b`):**
- The Embedding Coordinator (`domain/embedding.rs`), composing each node's input from its name plus the payload paths its type declares in `vector_search`, hashing it canonically, and calling the provider once per batch. Composition and embedding happen **before** the transaction, which is where DESIGN's ingest sequence puts them (step 5, ahead of step 6) and costs nothing: validation has already resolved every type record.
- The `embedding_space` table, and with it the identity half of `fr-embedding-dim-guard`. Boot compares the active provider's identity against the recorded one; on a mismatch the vector arm refuses (`EMBEDDING_SPACE_MISMATCH`, `failed_precondition`) instead of ranking across two spaces. Every other path is untouched, because only vectors are incomparable.
- The four vector states, on the `embedding_epoch` / `embedding_input_hash` columns that were previously written `NULL` unconditionally.
- Both producer-facing vector fields removed; `options.embed` added in their place.
- The in-process ONNX provider (`gears/graph-storage/onnx-embedding-plugin`), behind the gear's off-by-default `onnx` feature, verified against `all-MiniLM-L6-v2` rather than only compiled.

**Still deferred, and now the whole of what is left:**
- **The model-change lifecycle.** `requested → scanning → embedding → validating → cutover → complete`, its administrative API, the resumable backfill and per-tenant progress. Nothing opens a second epoch: a boot that finds a different identity reports it and blocks the arm, which is the safe half of the lifecycle without the recovery half. The gear has no background task of any kind, so this is a new capability rather than a missing branch.
- **The remote provider and its egress policy.** ADR-0004 requires a default-deny per-tenant policy over vendor, endpoint, region, data classes and vectorized fields before node text or user queries may leave a deployment. Building the plugin without that policy would be building the part that is easy to get wrong. *Superseded by D-025: the plugin is built; the policy is not.*
- **Chunk embeddings.** The `chunk` table is deferred (D-100), so "and every content chunk" has nothing to embed, and the "bounded content prefix" of the composed text is a prefix of name-plus-attributes only.
- **What readiness reports about the space, rather than the surface itself.** The surface is built (D-033): the embedding-space component answers `healthy`, or `unhealthy` with the mismatch named, the vector arm listed as what it blocks and re-embedding named as the recovery. What it does not carry is any *value* — the active identity and its dimension are not in the response, and neither is the number of rows left stale under the active epoch, which is the count D-027 asks for. `ComponentReadiness` holds a state and three prose fields and no numbers, so this is a shape question on the readiness DTO rather than a missing capability.

## D-104 [was deferred, now partly built — see D-030] Index-activation lifecycle, dynamic index DDL — and payload filtering entirely
`fr-index-admission` (PRD §5.1), ADR-0003 point 5 (`requested → building → active`, `CREATE INDEX CONCURRENTLY` worker, DDL queue keys). No runtime DDL, as recorded.

**Superseded in part by D-030.** Payload filtering and ordering are built; the index-activation lifecycle and per-path DDL are not, and cannot be from inside a gear today. The analysis below is kept as written because its two findings — B-tree over the extraction expression, scalar type resolved at registration — are what D-030 implements and what it still lacks.

**Corrected after checking.** The first draft said `$filter` over `index`-trait paths "is admitted only for paths covered by the static migration-time indexes", which implies payload filtering works for some paths. It works for none: the filterable-field schema declares `node_key`, `name`, `created_at` and `updated_at`, and nothing else, so `$filter=payload/severity eq 'critical'` is refused outright. Verified through the API. `index` trait paths *are* stored with the type's resolved traits — they are simply not wired to the filter surface, which is a larger gap than "no runtime DDL" and worth stating as its own deferral: **payload attributes are stored but not filterable at all**.

Checking this also found the rejection to be misclassified as `out_of_range`/`LIMIT_EXCEEDED`; fixed in `05695faa0`.

**Why it is not merely deferred: the platform binding blocks it regardless of what this gear builds.** Wiring the `index` trait to the filter surface is impossible in the platform OData binding as it stands, and no amount of index work changes that. Two places. `FilterField` declares its members as `const FIELDS: &'static [Self]` (`libs/toolkit-odata/src/filter.rs`), so the filterable set is fixed when the gear compiles — while a declared path belongs to a tenant's ontology and a type version, and its admissibility depends further on whether its index is active. And `FieldToColumn::map_field` returns a `Column`, with the predicate assembled as `Expr::col(column)` (`libs/toolkit-db/src/odata/sea_orm_filter.rs`), so a field must *be* a column; an extraction expression over one has nowhere to go. Both would be additive to fix: a field set resolved per request carrying its field kind, and a `map_field_expr(F) -> SimpleExpr` defaulting to today's behaviour. This entry is therefore part deferral, part platform gap.

**Two things the ADR left to the reader, and the projection cannot be built without either.**
- *A declared path needs a B-tree over its extraction expression, not a GIN over the payload.* Both are "a JSONB index", which is why it is easy to miss. The projection orders and paginates by keyset, so `$filter`, `$orderby` and the cursor all need a total order over the filtered field; GIN answers containment and existence. One GIN covers equality over every path and ordering over none — so the v1 prototype's single GIN was not a cheaper form of the decision but a much weaker one, admitting `eq` and leaving every ordered projection to a full scan and a sort with nothing to say so.
- *A declared path needs a resolved scalar type.* The trait is a list of pointers carrying no type, which is right — the pointer already points into the type's own schema. But the resolution must be explicit, because the index expression needs the cast (`->>` yields `text`), comparison needs the semantics (`'10' < '9'`), the cursor codec needs the field kind, and something must happen when a payload holds an object where the path was declared scalar. Registration should reject a path that does not land on a scalar rather than build an index nothing will use.

- **Folded into the docs:** ADR-0003 now names the index kind in the decision itself, carries both findings as consequences marked "Found while building the prototype", and gains a "What payload filtering needs from the platform" section in the form ADR-0005 uses — two numbered asks, recorded as *unraised*, because the lifecycle they would serve is unimplemented and there is nothing yet to measure a proposed signature against. DESIGN's trait table row now says B-tree over the path expression, `$filter` **and** `$orderby`, scalar required. Published in `215380db9`.
- **Not changed, deliberately:** the `index` trait's own `description` in the base node schema still says "backed by a JSONB index and admissible in `$filter`". Correcting it is a base-schema edit, which has no delivery path on an existing database — see D-013.

## D-105 [deferred] Full admission layer (fairness, queues, reserved connections)
`nfr-tenant-fairness`, parts of the Capacity contract (`tenant_max_*`, `global_max_*`, `interactive_reserved_connections`). This iteration ships the per-request bounds subset (sizes, depths, budgets, page caps) enforced in the domain admission layer.

## D-106 [deferred] Tenant offboarding / deletion monotonicity
`fr-tenant-offboarding` (PRD §5.8), the six-step external-ledger protocol.

## D-107 [deferred] Analytics topology role and metric annotation
`fr-analytics-topology`, `fr-metric-annotation` (PRD §5.7), ADR-0007 grants. The `graph_revision` signal (`fr-revision-signal`) IS in scope.

## D-108 [deferred] PG16 configuration matrix
ADR-0001 point 2 makes PG16+ the baseline and demands CI on both PG16 and PG19. This iteration pins the test lane to PG19 (`postgres_graph()`, `19beta3-alpine`); the server-major probe and conditional property-graph DDL are implemented, but the PG16 lane is not exercised.

## D-109 [was deferred, now built — see D-033] The readiness surface, entirely
`fr-readiness`: DESIGN's 14-row matrix is normative, and `GET /health/ready` is in its REST surface.

**Corrected after checking.** The first draft said this iteration "ships per-capability healthy/degraded/unhealthy (DB, SQL/PGQ availability, vector dimension check) without the full matrix semantics". It ships none of it: `GET /health/ready` answers 404 and the word `health` does not appear in the route registration. What exists are the *inputs* a readiness surface would report — the capability probe, the embedding-dimension check at boot — with nothing exposing them. The deferral is the whole surface, not its fidelity.

## D-110 [deferred] Observability contract
`fr-observability` deny-by-default telemetry allowlist: followed in spirit (no payload/query text in logs), but the metric/counter surface (saturation counters, high-watermark gauges per limit) is not built.

---

# What the upstream contribution carries

A reviewer of the upstream PR does not need all fifty-odd entries; they need to
know what is open, and in what order it would hurt. Ranked by release risk —
security and silent-wrong-answer first, then behaviour narrower than the
documents describe, then what waits on the platform. Each line names the entry
that carries the detail.

**Security, and answers that are wrong without saying so.**

1. **The ingest producer principal is never carried into the store** (D-039) — the scope fence's owning-producer check compares an empty string with itself, so any writer in a tenant can replace any producer's scope, and the idempotency key is tenant- rather than tenant-and-producer-scoped. *Being fixed separately; a p1 authorization gap, not a deferral.*
2. **A tombstoned edge is silently revived by the ordinary write path** (D-049) — a re-sync resurrects a deliberately deleted relationship and erases its tombstone audit; deleting an already-deleted row answers 404 where the contract says no-op. *Behaviour, not a deferral.*
3. **A traversal hop that reaches the edge-scan budget truncates silently** (D-044) — `TruncationReason::EdgeScanCap` exists and nothing ever emits it, so an arbitrarily cut subgraph is reported as complete; the budget is also per hop rather than cumulative. *Behaviour.*
4. **The authorizing permission's type pattern is not intersected with the caller's** (D-046) — DESIGN specifies an effective type set from both; the enforcer yields only an `AccessScope`, and type-family filtering has no test at all. *Half unimplemented, half untested.*
5. **`nfr-response-bound` has no configuration key and nothing truncates before hydration** (D-041) — a page of individually admissible rows is unbounded in aggregate, which is a denial-of-service surface reachable with valid requests. *A p1 requirement with no implementation.*
6. **An unconfigured deployment gets the deterministic fake embedding provider** (D-019) — vector search then answers well-formed, semantically arbitrary rankings, warned about only in a boot log. *A prototype default that must not ship as one.*
7. **The remote embedding provider ships without the default-deny per-tenant egress policy ADR-0004 puts in front of it** (D-025) — selecting `remote` sends every tenant's node and query text to one configured endpoint. *A prototype trade, stated as such.*

**Narrower than the documents describe, and correct within its narrower shape.**

8. **Writes are one statement per row** where `fr-bulk-ingest` requires batched statements (D-040) — inside the published budgets today, on the slow part of a non-linear curve for large graphs.
9. **The neighborhood projection truncates by arrival order, not by degree** (D-042) — well-formed, bounded, and keeps the wrong nodes.
10. **Traversal takes explicit seed keys only, reports no seed list or admitted-seed count, and bounds the raw seed set before dedup and authorization** (D-043) — the search-then-expand scenario is two calls the caller joins.
11. **The deadline is minted per store call rather than per request, and no store read consults it** (D-052) — what actually kills a slow request is the gateway's 30 s ceiling.
12. **A search hit carries no element envelope** (D-045) — defensible while a hit is a payload-free reference, undecided in writing.
13. **`idempotency_retention_days` is a key nothing reads** (D-047) and **`nfr-code-coverage`'s 85 % is gated by CI at 80 %** (D-048) — two documented enforcements that do not enforce.
14. **The built-in store declines the one-snapshot obligation** (D-007) and **the tabular projection reports its revision per row rather than per page** (D-005, D-021) — both declared through the capability mechanism rather than hidden.
15. **Base schemas have no upgrade path once published** (D-013) — a base-schema edit, editorial ones included, can never reach a database that has already published it.
16. **Lexical search cannot find identifiers inside file names** (D-028) — the default parser's token classes, fixable in the composed search text.
17. **Payload filtering and ordering are built without the index-activation lifecycle** (D-030, D-104) — equality is indexed, range and order read the extraction expression; a declared path is filterable the moment it is declared, with no capacity check.
18. **Scope replacement's row ceilings exist only in the built-in store** (D-034) — the conformance suite covers the grounds on both implementations and the bounds on neither.

**Agreed scope cuts, shipped as gaps rather than as surprises.**

19. Labels, entirely — and the whole of the port refuses in one voice (D-101).
20. Content chunking and heavy-content offload (D-100); change events via the transactional outbox (D-102); tenant offboarding (D-106); analytics topology role and metric annotation (D-107); the fairness and queueing half of the admission layer (D-105); the observability counter surface (D-110).
21. The embedding model-change lifecycle and its resumable backfill (D-103) — the safe half (refuse on identity mismatch) without the recovery half.
22. The retained-revision history behind type evolution, and a backfill for what a migration marked stale (D-031) — deferred by decision, and waiting with two other needs on a background task the gear does not have.
23. No epoch-rotation mechanism, so `fr-snapshot-identity` holds for idempotency receipts only (D-050).

**Platform gaps — nothing in this gear closes them.**

24. **No row-locking surface in the secure ORM** (D-035) — the fence is an upsert because `SELECT … FOR UPDATE` is unreachable from a gear; the next gear that needs one will write the same three broken steps.
25. **No DDL surface a gear may call** (D-104), which is what blocks per-path indexes and the activation lifecycle above them.
26. **No per-request OData field set and no field-to-expression mapping** (D-104) — the gear renders the plan itself meanwhile.
27. **No revision slot on `CursorV1`/`PageInfo`** (D-005) — closed for this gear by putting the revision on the element (D-021), still true of the platform.
28. **No PG19 + pgvector test image** (D-003) and **no PG16 lane** (D-108); **readiness cannot name the required server major** (D-004), a diagnostics gap rather than a correctness one.
29. **No platform health surface for the PEP**, so the authz row of the readiness matrix is reported absent rather than healthy (D-033).
30. **No startup publication to the platform types-registry** (D-051) — the gear keeps only its own per-tenant projection, so producers cannot browse the base ontology to derive from it and administrators cannot grant permissions over types the registry has never seen.
